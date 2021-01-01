use crate::fs::FsWatcher;
use crate::ws::server::WsServer;
use actix::{Actor, Addr, Context, Handler, Message};
use bs3_files::served::Served;

pub struct Server {
    pub ws_server: Addr<WsServer>,
    pub fs_server: Addr<FsWatcher>,
    pub served_files: Addr<Served>,
    pub port: Option<u16>,
    pub bind_address: String,
}

impl Actor for Server {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        println!("yay! running now!!!!!!!");
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "String")]
pub enum ServerIncoming {
    Stop,
}

impl Handler<ServerIncoming> for Server {
    type Result = String;

    fn handle(&mut self, msg: ServerIncoming, ctx: &mut Context<Self>) -> Self::Result {
        println!("Received a STOP message...");
        String::from("hahaha")
    }
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Server")
            .field("ws_server", &String::from("Addr<WsServer>"))
            .field("fs_server", &String::from("Addr<FsWatcher>"))
            .field("served_files", &String::from("Addr<Served>"))
            .field("port", &self.port)
            .field("bind_address", &self.bind_address)
            .finish()
    }
}
