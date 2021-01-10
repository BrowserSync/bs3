#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use typescript_definitions::TypescriptDefinition;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypescriptDefinition)]
#[serde(tag = "kind", content = "payload")]
pub enum BrowserSyncOutputMsg {
    Listening { bind_address: String },
}

#[cfg(not(target_arch = "wasm32"))]
impl actix::Message for BrowserSyncOutputMsg {
    type Result = ();
}
