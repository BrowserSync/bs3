pub mod msg;

use crate::output::msg::BrowserSyncOutputMsg;

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

#[cfg(not(target_arch = "wasm32"))]
impl actix::Actor for StdOut {
    type Context = actix::Context<Self>;
}

#[cfg(not(target_arch = "wasm32"))]
impl actix::Handler<BrowserSyncOutputMsg> for StdOut {
    type Result = ();

    fn handle(
        &mut self,
        msg: BrowserSyncOutputMsg,
        _ctx: &mut actix::Context<Self>,
    ) -> Self::Result {
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
