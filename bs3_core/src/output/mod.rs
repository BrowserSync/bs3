use crate::start::BrowserSyncOutputMsg;
use actix::{Actor, Context, Handler};

pub struct StdOut {
    format: StdOutFormat,
}

impl Default for StdOut {
    fn default() -> Self {
        Self {
            format: StdOutFormat::JsonPretty,
        }
    }
}

enum StdOutFormat {
    Human,
    Json,
    JsonPretty,
}

impl Actor for StdOut {
    type Context = Context<Self>;
}

impl Handler<BrowserSyncOutputMsg> for StdOut {
    type Result = ();

    fn handle(&mut self, msg: BrowserSyncOutputMsg, _ctx: &mut Context<Self>) -> Self::Result {
        match self.format {
            StdOutFormat::Human => match msg {
                BrowserSyncOutputMsg::Listening { bind_address } => {
                    println!("Server ready at {}", bind_address);
                }
            },
            StdOutFormat::Json => {
                println!(
                    "{}",
                    serde_json::to_string(&msg).expect("msg can always be serialized")
                )
            }
            StdOutFormat::JsonPretty => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&msg).expect("msg can always be serialized")
                )
            }
        }
    }
}
