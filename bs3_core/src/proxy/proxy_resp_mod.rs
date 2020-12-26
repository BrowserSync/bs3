use crate::resp::RespMod;
use actix_web::dev::{RequestHead, ResponseHead};
use actix_web::http::HeaderMap;

#[derive(Debug, Clone)]
pub struct ProxyResp {
    pub target_url: url::Url,
    pub local_url: url::Url,
}

impl RespMod for ProxyResp {
    fn process_str(&self, str: String) -> String {
        let target = format!(
            "{}://{}",
            self.target_url.scheme(),
            self.target_url.host_str().expect("")
        );
        let local = format!(
            "{}://{}:{}",
            self.local_url.scheme(),
            self.local_url.host_str().expect(""),
            self.local_url.port().expect("local has a port")
        );
        log::trace!("replace [{}] with [{}]", target, local);
        str.replace(&target, &local)
    }
    fn guard(&self, req_head: &RequestHead, res_head: &ResponseHead) -> bool {
        is_accept_html(&req_head.headers) && is_content_type_html(&res_head.headers)
    }
    fn name(&self) -> String {
        String::from("proxy resp mod")
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
