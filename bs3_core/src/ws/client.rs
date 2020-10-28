#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use std::path::PathBuf;
use typescript_definitions::TypescriptDefinition;

#[derive(
    Default,
    Clone,
    Debug,
    Eq,
    Hash,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
    TypescriptDefinition,
)]
pub struct ServedFile {
    pub path: PathBuf,
    pub web_path: PathBuf,
    pub referer: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
impl actix::Message for ServedFile {
    type Result = ();
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypescriptDefinition)]
#[serde(tag = "kind")]
pub enum ClientMsg {
    Connect,
    Disconnect,
    Scroll(ScrollMsg),
    FsNotify(FsNotify),
}

#[cfg(not(target_arch = "wasm32"))]
impl actix::Message for ClientMsg {
    type Result = ();
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypescriptDefinition)]
pub struct FsNotify {
    pub item: ServedFile,
}

#[cfg(not(target_arch = "wasm32"))]
impl actix::Message for FsNotify {
    type Result = ();
}

impl FsNotify {
    pub fn new(item: impl Into<ServedFile>) -> Self {
        Self { item: item.into() }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, TypescriptDefinition)]
pub struct ScrollMsg {
    pub x: f64,
    pub y: f64,
}

#[test]
fn test_client_msg() {
    let js = serde_json::json!({
        "kind": "Scroll",
        "x": 0,
        "y": -100
    });
    let msg: ClientMsg = serde_json::from_value(js).expect("test");
    println!("{:?}", msg);
}
