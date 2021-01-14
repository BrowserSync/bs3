pub mod stop;

use crate::browser_sync::BrowserSync;
use crate::bs_error::BsError;
use actix::{Actor, Context, Handler, Message, Recipient};
use actix_web::http::StatusCode;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};

use crate::output::msg::BrowserSyncOutputMsg;
use crate::routes::msg::incoming_msg;
use actix_rt::time::delay_for;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use wasm_bindgen::__rt::std::sync::Arc;

#[derive(Default)]
pub struct Server {
    // pub ws_server: Addr<WsServer>,
    // pub fs_server: Addr<FsWatcher>,
    // pub served_files: Addr<Served>,
    // pub port: Option<u16>,
    // pub bind_address: String,
    pub output_recipients: Vec<Recipient<BrowserSyncOutputMsg>>,
}

impl Actor for Server {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        log::trace!("main server started")
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
    pub output_recipients: Option<Vec<Recipient<BrowserSyncOutputMsg>>>,
}

impl Handler<Start> for Server {
    type Result = Pin<Box<dyn Future<Output = ()>>>;

    fn handle(&mut self, msg: Start, _ctx: &mut Context<Self>) -> Self::Result {
        log::trace!("got start msg for address {}", msg.bs.bind_address());

        // if the start message contains a recipient, add it to the locally saved ones
        if let Some(recipients) = msg.output_recipients.as_ref() {
            self.output_recipients.extend(recipients.clone());
        }

        let output_recipients = self.output_recipients.clone();
        let port_num = msg
            .bs
            .local_url
            .0
            .port()
            .expect("port MUST be defined here");
        let exec = async move {
            let (stop_sender, mut stop_recv) = tokio::sync::mpsc::channel::<()>(1);
            let as_arc = Arc::new(Mutex::new(stop_sender));

            let server = HttpServer::new(move || {
                App::new().data(as_arc.clone()).service(welcome).service(
                    web::resource("/__bs/incoming_msg").route(web::post().to(incoming_msg)),
                )
            });
            let server = server
                .bind(msg.bs.bind_address())
                .map_err(|e| BsError::CouldNotBind {
                    e: anyhow::anyhow!(e),
                    port: port_num,
                });
            match server {
                Ok(server) => {
                    output_recipients.iter().for_each(|recipient| {
                        let sent = recipient.do_send(BrowserSyncOutputMsg::Listening {
                            bind_address: msg.bs.bind_address(),
                        });
                        if let Err(sent_err) = sent {
                            eprintln!("could not send binding message {}", sent_err);
                        }
                    });
                    let s = server.run();
                    let s2 = s.clone();
                    actix_rt::spawn(async move {
                        while let Some(msg) = stop_recv.recv().await {
                            println!("got a stop");
                            println!("sending a stop message...");
                            // delay_for(std::time::Duration::from_secs(1)).await;
                            s2.stop(true);
                        }
                    });
                    match s.await.map_err(BsError::unknown) {
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
