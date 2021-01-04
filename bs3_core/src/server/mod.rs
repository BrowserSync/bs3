use crate::bs_error::BsError;
use actix::{Actor, AsyncContext, Context, Handler, Message, SpawnHandle, SyncArbiter, WrapFuture};
use actix_web::http::StatusCode;
use actix_web::{App, HttpRequest, HttpResponse, HttpServer};
use std::future::Future;
use std::pin::Pin;
use tokio::sync::oneshot::Receiver;

pub struct Server {
    // pub ws_server: Addr<WsServer>,
    // pub fs_server: Addr<FsWatcher>,
    // pub served_files: Addr<Served>,
    // pub port: Option<u16>,
    // pub bind_address: String,
    pub(crate) spawn_handle: Option<SpawnHandle>,
}

impl Actor for Server {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("yay! running now!!!!!!!");
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Ping;

impl Handler<Ping> for Server {
    type Result = ();

    fn handle(&mut self, msg: Ping, ctx: &mut Context<Self>) -> Self::Result {
        println!("got ping");
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Start {
    pub bind: String,
}

impl Handler<Start> for Server {
    type Result = Pin<Box<dyn Future<Output = ()>>>;

    fn handle(&mut self, msg: Start, ctx: &mut Context<Self>) -> Self::Result {
        println!("got start msg");
        let (tx, rx) = tokio::sync::oneshot::channel::<()>();
        let exec = async move {
            let server = HttpServer::new(move || App::new().service(welcome));
            let server = server
                .bind(msg.bind.clone())
                .map_err(|e| BsError::CouldNotBind {
                    e: anyhow::anyhow!(e),
                    port: 8080,
                });
            if let Ok(server) = server {
                println!("running...");
                match server.run().await.map_err(BsError::unknown) {
                    Ok(_) => {
                        println!("server All done");
                        tx.send(());
                    }
                    Err(e) => {
                        println!("server error e={}", e);
                        tx.send(());
                    }
                }
            }
        };
        ctx.spawn(exec.into_actor(self));
        Box::pin(async move {
            rx.await;
        })
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "String")]
pub enum ServerIncoming {
    Stop,
}

impl Handler<ServerIncoming> for Server {
    type Result = String;

    fn handle(&mut self, _msg: ServerIncoming, _ctx: &mut Context<Self>) -> Self::Result {
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
            // .field("port", &self.port)
            // .field("bind_address", &self.bind_address)
            .finish()
    }
}

#[actix_web::get("/")]
async fn welcome(_req: HttpRequest) -> actix_web::Result<HttpResponse> {
    // response
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body("hello world"))
}
