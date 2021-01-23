pub mod stop;

use crate::browser_sync::BrowserSync;
use crate::bs_error::BsError;
use actix::{Actor, AsyncContext, Context, Handler, Message, Recipient};
use actix_web::http::StatusCode;
use actix_web::{guard, web, App, HttpRequest, HttpResponse, HttpServer};

use crate::output::msg::BrowserSyncOutputMsg;
use crate::routes::gql::{gql_playgound, gql_response, GQL_ENDPOINT};
use crate::routes::gql_mutation::MutationRoot;
use crate::routes::gql_query::{BrowserSyncGraphData, QueryRoot};

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
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct Start {
    pub bs: BrowserSync,
}

impl Start {
    pub fn new(bs: BrowserSync) -> Self {
        Self { bs }
    }
}

impl Handler<Start> for Server {
    type Result = Pin<Box<dyn Future<Output = Result<(), anyhow::Error>>>>;

    fn handle(&mut self, msg: Start, ctx: &mut Context<Self>) -> Self::Result {
        log::trace!("got start msg for address {}", msg.bs.bind_address());
        let self_addr = ctx.address();
        let bind_address_clone = msg.bs.bind_address().clone();

        {
            let mut i = self.bs_instances.lock().unwrap();
            i.push(msg.bs.clone());
        }

        // if the start message contains a recipient, add it to the locally saved ones
        // if let Some(recipients) = msg.output_recipients.as_ref() {
        //     self.output_recipients.extend(recipients.clone());
        // }
        // let output_recipients = self.output_recipients.clone();

        let bs_instances_arc = self.bs_instances.clone();

        let exec = async move {
            let port_num = msg
                .bs
                .local_url
                .0
                .port()
                .expect("port MUST be defined here");
            let (stop_sender, mut stop_recv) = tokio::sync::mpsc::channel::<()>(1);
            let stop_msg = Arc::new(tokio::sync::Mutex::new(stop_sender));
            let schema: Schema<QueryRoot, MutationRoot, EmptySubscription> =
                Schema::build(QueryRoot, MutationRoot, EmptySubscription)
                    .data(BrowserSyncGraphData {
                        bs_instances: bs_instances_arc.clone(),
                    })
                    .data(stop_msg.clone())
                    .finish();

            let server = HttpServer::new(move || {
                App::new()
                    .data(schema.clone())
                    .data(stop_msg.clone())
                    .service(
                        web::resource(GQL_ENDPOINT)
                            .guard(guard::Post())
                            .to(gql_response),
                    )
                    .service(
                        web::resource(GQL_ENDPOINT)
                            .guard(guard::Get())
                            .to(gql_playgound),
                    )
                    .service(welcome)
            });
            let server = server
                .disable_signals()
                .bind(msg.bs.bind_address())
                .map_err(|e| BsError::could_not_bind(port_num, e))?;

            // output_recipients.iter().for_each(|recipient| {
            //     let sent = recipient.do_send(BrowserSyncOutputMsg::Listening {
            //         bind_address: msg.bs.bind_address(),
            //     });
            //     if let Err(sent_err) = sent {
            //         eprintln!("could not send binding message {}", sent_err);
            //     }
            // });

            let s = server.run();
            let s2 = s.clone();

            self_addr.do_send(NotifyRecipientsMsg {
                messages: vec![BrowserSyncOutputMsg::Listening {
                    bind_address: msg.bs.bind_address(),
                }],
            });

            actix_rt::spawn(async move {
                while let Some(_msg) = stop_recv.recv().await {
                    println!("got a stop");
                    println!("sending a stop message...");
                    // delay_for(std::time::Duration::from_secs(1)).await;
                    s2.stop(false).await;
                    self_addr.do_send(RemoveInstance {
                        bind_address: bind_address_clone.clone(),
                    });
                }
            });
            match s.await.map_err(BsError::unknown) {
                Ok(_) => {
                    println!("server all done on port {}", port_num);
                }
                Err(e) => {
                    println!("server error on port {}", port_num);
                    eprintln!("\t{:?}", e);
                }
            };
            Ok(())
        };
        Box::pin(exec)
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct RegisterRecipientMsg {
    pub output_recipients: Vec<Recipient<BrowserSyncOutputMsg>>,
}

impl RegisterRecipientMsg {
    pub fn new(recipient: Recipient<BrowserSyncOutputMsg>) -> Self {
        RegisterRecipientMsg {
            output_recipients: vec![recipient],
        }
    }
}

impl Handler<RegisterRecipientMsg> for Server {
    type Result = ();

    fn handle(&mut self, msg: RegisterRecipientMsg, _ctx: &mut Context<Self>) -> Self::Result {
        self.output_recipients.extend(msg.output_recipients);
    }
}

#[derive(Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct NotifyRecipientsMsg {
    pub messages: Vec<BrowserSyncOutputMsg>,
}

impl Handler<NotifyRecipientsMsg> for Server {
    type Result = ();

    fn handle(&mut self, msg: NotifyRecipientsMsg, _ctx: &mut Context<Self>) -> Self::Result {
        self.output_recipients.iter().for_each(|recipient| {
            msg.messages.iter().for_each(|msg| {
                let sent = recipient.do_send(msg.clone());
                if let Err(sent_err) = sent {
                    eprintln!("could not send binding message {}", sent_err);
                }
            });
        });
    }
}

// fn get() -> Pin<Box<impl Future<Output = Result<(), anyhow::Error>>>> {
//     Box::pin(async move { Ok(()) })
// }

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
