use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use rand::{self, rngs::ThreadRng, Rng};

#[derive(Default, Debug)]
pub struct Served {
    pub items: HashSet<ServedFile>,
    pub listeners: HashMap<usize, Recipient<ServedFile>>,
    pub rng: ThreadRng,
}

impl Actor for Served {
    type Context = Context<Self>;
}

#[derive(Message, Default, Clone, Debug, Eq, Hash, PartialEq)]
#[rtype(result = "()")]
pub struct ServedFile {
    pub path: PathBuf,
    pub web_path: PathBuf,
    pub referer: Option<String>,
}

impl Handler<ServedFile> for Served {
    type Result = ();

    fn handle(&mut self, msg: ServedFile, _ctx: &mut Context<Self>) -> Self::Result {
        self.items.insert(msg.clone());
        log::debug!("self.items contains {} entries", self.items.len());
        log::debug!("relaying this msg to {} listeners", self.listeners.len());
        for (_id, listener) in self.listeners.iter() {
            if let Err(_e) = listener.do_send(msg.clone()) {
                log::error!("failed to send ServedFile to a listener");
            }
        }
    }
}

#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Register {
    pub addr: Recipient<ServedFile>,
}
impl Handler<Register> for Served {
    type Result = ();

    fn handle(&mut self, msg: Register, _ctx: &mut Context<Self>) -> Self::Result {
        let Register { addr } = msg;
        let id = self.rng.gen::<usize>();
        self.listeners.insert(id, addr);
        log::debug!("self.listeners {:#?}", self.listeners);
    }
}

///
/// This is a wrapper to enabled easy sharing of the Addr of Served
///
#[derive(Debug)]
pub struct ServedAddr(pub Addr<Served>);
