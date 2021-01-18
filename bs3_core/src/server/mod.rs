pub mod stop;

use crate::browser_sync::BrowserSync;
use crate::bs_error::BsError;
use actix::{Actor, AsyncContext, Context, Handler, Message, Recipient};
use actix_web::http::StatusCode;
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer};

use crate::output::msg::BrowserSyncOutputMsg;
use crate::routes::gql::{gql_playgound, gql_response};
use crate::routes::gql_mutation::MutationRoot;
use crate::routes::gql_query::{BrowserSyncGraphData, QueryRoot};
use crate::routes::msg::incoming_msg;

use async_graphql::{EmptySubscription, Schema};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct Server {
    // pub ws_server: Addr<WsServer>,
    // pub fs_server: Addr<FsWatcher>,
    // pub served_files: Addr<Served>,
    // pub port: Option<u16>,
    // pub bind_address: String,
    pub output_recipients: Vec<Recipient<BrowserSyncOutputMsg>>,
    pub bs_instances: Arc<Mutex<Vec<BrowserSync>>>,
}

impl Actor for Server {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        log::trace!("main server started")
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct RemoveInstance {
    bind_address: String,
}

impl Handler<RemoveInstance> for Server {
    type Result = ();

    fn handle(&mut self, msg: RemoveInstance, _ctx: &mut Context<Self>) -> Self::Result {
        let mut addresses = self.bs_instances.lock().unwrap();
        let index = addresses
            .iter()
            .position(|bs| bs.bind_address() == msg.bind_address);
        if let Some(addr) = index {
            addresses.remove(addr);
        }
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

    fn handle(&mut self, msg: Start, ctx: &mut Context<Self>) -> Self::Result {
        log::trace!("got start msg for address {}", msg.bs.bind_address());
        let self_addr = ctx.address();
        let self_addr_clone = self_addr.clone();
        let bind_address_clone = msg.bs.bind_address().clone();

        {
            let mut i = self.bs_instances.lock().unwrap();
            i.push(msg.bs.clone());
        }

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
        let arc = self.bs_instances.clone();
        let exec = async move {
            let (stop_sender, mut stop_recv) = tokio::sync::mpsc::channel::<()>(1);
            let stop_msg = Arc::new(tokio::sync::Mutex::new(stop_sender));
            let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
                .data(BrowserSyncGraphData {
                    bs_instances: arc.clone(),
                })
                .finish();

            let server = HttpServer::new(move || {
                App::new()
                    .data(schema.clone())
                    .data(stop_msg.clone())
                    .service(
                        web::resource("/__bs/graphql")
                            .guard(guard::Post())
                            .to(gql_response),
                    )
                    .service(
                        web::resource("/__bs/graphql")
                            .guard(guard::Get())
                            .to(gql_playgound),
                    )
                    .service(welcome)
                    .service(
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
                        while let Some(_msg) = stop_recv.recv().await {
                            println!("got a stop");
                            println!("sending a stop message...");
                            // delay_for(std::time::Duration::from_secs(1)).await;
                            s2.stop(true).await;
                            self_addr_clone.do_send(RemoveInstance {
                                bind_address: bind_address_clone.clone(),
                            });
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
