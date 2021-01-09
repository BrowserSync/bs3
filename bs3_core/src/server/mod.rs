use crate::browser_sync::BrowserSync;
use crate::bs_error::BsError;
use actix::{Actor, Context, Handler, Message};
use actix_web::http::StatusCode;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};

use crate::config::Config;
use std::future::Future;
use std::pin::Pin;

pub struct Server {
    // pub ws_server: Addr<WsServer>,
// pub fs_server: Addr<FsWatcher>,
// pub served_files: Addr<Served>,
// pub port: Option<u16>,
// pub bind_address: String,
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

    fn handle(&mut self, _msg: Ping, _ctx: &mut Context<Self>) -> Self::Result {
        println!("got ping");
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct Start {
    pub bs: BrowserSync,
}

/// This handler uses json extractor with limit
async fn extract_item(item: web::Json<Config>, req: HttpRequest) -> HttpResponse {
    println!("request: {:?}", req);
    let inner = item.into_inner();
    println!("model: {:?}", inner);

    HttpResponse::Ok().json(inner) // <- send json response
}

impl Handler<Start> for Server {
    type Result = Pin<Box<dyn Future<Output = ()>>>;

    fn handle(&mut self, msg: Start, _ctx: &mut Context<Self>) -> Self::Result {
        println!("got start msg for address {}", msg.bs.bind_address());
        let port_num = msg.bs.config.port.expect("port MUST be defined here");
        let exec = async move {
            let server = HttpServer::new(move || {
                App::new()
                    .service(welcome)
                    .service(web::resource("/__bs").route(web::post().to(extract_item)))
            });
            let server = server
                .bind(msg.bs.bind_address())
                .map_err(|e| BsError::CouldNotBind {
                    e: anyhow::anyhow!(e),
                    port: port_num,
                });
            match server {
                Ok(server) => {
                    println!("running...");
                    match server.run().await.map_err(BsError::unknown) {
                        Ok(_) => {
                            println!("server All done");
                        }
                        Err(e) => {
                            println!("server error e={}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("error from bind ||||{}||||", e);
                }
            }
        };
        Box::pin(exec)
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
