use crate::resp::RespMod;
use actix_web::dev::{RequestHead, ResponseHead};

#[derive(Debug, Clone)]
pub struct Css;

impl RespMod for Css {
    fn process_str(&self, str: String) -> String {
        let mut output = String::from(str);
        output.push_str("/* hello */");
        output
    }
    fn guard(&self, _req_head: &RequestHead, res_head: &ResponseHead) -> bool {
        res_head
            .headers
            .get("accept")
            .and_then(|hv| hv.to_str().ok())
            .filter(|str| str.contains("text/css"))
            .is_some()
    }
    fn name(&self) -> String {
        String::from("CSS comment")
    }
}
