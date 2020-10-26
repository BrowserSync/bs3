use crate::fs::FsNotify;
use actix::Message;

#[derive(Debug, Clone, Message, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind")]
#[rtype(result = "()")]
pub enum ClientMsg {
    Connect,
    Disconnect,
    Scroll(ScrollMsg),
    FsNotify(FsNotify),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScrollMsg {
    pub x: isize,
    pub y: isize,
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
