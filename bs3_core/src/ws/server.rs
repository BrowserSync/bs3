use actix::prelude::*;

use crate::ws::client::{ClientMsg, FsNotify};
use rand::{self, rngs::ThreadRng, Rng};
use std::collections::{HashMap, HashSet};

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<ClientMsg>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Send message to specific room
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct ClientBroadcastMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: ClientMsg,
    /// Room name
    pub room: String,
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client id
    pub id: usize,
    /// Room name
    pub name: String,
}

/// `ChatServer` manages chat rooms and responsible for coordinating chat
/// session. implementation is super primitive
pub struct WsServer {
    sessions: HashMap<usize, Recipient<ClientMsg>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
}

impl Default for WsServer {
    fn default() -> WsServer {
        // default room
        let mut rooms = HashMap::new();
        rooms.insert("Main".to_owned(), HashSet::new());

        WsServer {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
        }
    }
}

impl WsServer {
    /// Send message to all users in the room
    fn send_message(&self, room: &str, message: ClientMsg, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        let _ = addr.do_send(message.clone());
                    } else {
                        ()
                    }
                }
            }
        }
    }
}

/// Make actor from `ChatServer`
impl Actor for WsServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::debug!("stopped!");
    }
}

impl Handler<FsNotify> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: FsNotify, _ctx: &mut Context<Self>) -> Self::Result {
        let msg = ClientMsg::FsNotify(msg);
        self.send_message(&"Main".to_owned(), msg, 0);
    }
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for WsServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // notify all users in same room
        self.send_message(&"Main".to_owned(), ClientMsg::Connect, 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        log::trace!("+ client connected = ({})", id);
        self.sessions.insert(id, msg.addr);

        log::trace!("rooms before={:?}", self.rooms);
        // auto join session to Main room
        self.rooms
            .entry("Main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        log::trace!("rooms after={:?}", self.rooms);

        // send id back
        id
    }
}

impl Handler<Disconnect> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        log::trace!("- client disconnected {}", msg.id);

        let mut rooms: Vec<String> = Vec::new();

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message(&room, ClientMsg::Disconnect, 0);
        }
    }
}

/// Handler for Message message.
impl Handler<ClientBroadcastMessage> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: ClientBroadcastMessage, _: &mut Context<Self>) {
        self.send_message(&msg.room, msg.msg, msg.id);
    }
}

/// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for WsServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, name } = msg;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }
        // send message to other users
        for room in rooms {
            self.send_message(&room, ClientMsg::Disconnect, 0);
        }

        self.rooms
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        self.send_message(&name, ClientMsg::Connect, id);
    }
}
