use crate::resp::RespMod;
use actix_web::dev::{RequestHead, ResponseHead};

#[derive(Debug, Clone)]
pub struct Css;

impl RespMod for Css {
    fn name(&self) -> String {
        String::from("Chunked test")
    }
    fn process_str(&self, str: String) -> String {
        str.replace("Chunked", "[Chunked]")
    }
    fn guard(&self, req_head: &RequestHead, res_head: &ResponseHead) -> bool {
        req_head.uri
            .clone()
            .into_parts()
            .path_and_query
            .map(|pq| pq.as_str().starts_with("/chunked"))
            .unwrap_or(false)
    }
}
