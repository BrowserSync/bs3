use bytes::Bytes;
use actix_web::dev::{RequestHead, ResponseHead};
use crate::resp::{RespMod, RespGuard};

#[derive(Debug, Clone)]
pub struct Script;

impl RespMod for Script {
    fn process_str(&self, str: String) -> Bytes {
        Bytes::from(str.replace("</body>", "<script>console.log('here!')</script></body>"))
    }
}

impl RespGuard for Script {
    fn check(&self, req_head: &RequestHead) -> bool {
        if req_head.headers.contains_key("accept") {
            if req_head.headers.get("accept")
                .expect("guarded")
                .to_str()
                .expect("ed")
                .contains("text/html") {
                return true;
            } else {
                println!("not doing {:#?}", req_head.uri)
            }
        }
        return false;
    }
}
