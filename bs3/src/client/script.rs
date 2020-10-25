use crate::resp::RespMod;
use actix_web::dev::{RequestHead, ResponseHead};
use actix_web::http::HeaderMap;

#[derive(Debug, Clone)]
pub struct Script;

impl RespMod for Script {
    fn process_str(&self, str: String) -> String {
        str.replace("</body>", "<script>console.log('here!')</script></body>")
    }
    fn guard(&self, req_head: &RequestHead, res_head: &ResponseHead) -> bool {
        has_no_referer(&req_head.headers)
            && is_accept_html(&req_head.headers)
            && is_content_type_html(&res_head.headers)
    }
    fn name(&self) -> String {
        String::from("bs3 script tag")
    }
}

fn is_accept_html(headers: &HeaderMap) -> bool {
    headers
        .get("accept")
        .and_then(|hv| hv.to_str().ok())
        .filter(|str| str.contains("text/html"))
        .is_some()
}

fn is_content_type_html(headers: &HeaderMap) -> bool {
    headers
        .get("content-type")
        .and_then(|hv| hv.to_str().ok())
        .filter(|str| str.contains("text/html"))
        .is_some()
}

fn has_no_referer(headers: &HeaderMap) -> bool {
    headers.get("referer").is_none()
}
