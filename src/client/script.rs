use crate::resp::RespMod;
use actix_web::dev::RequestHead;

#[derive(Debug, Clone)]
pub struct Script;

impl RespMod for Script {
    fn process_str(&self, str: String) -> String {
        str.replace("</body>", "<script>console.log('here!')</script></body>")
    }
    fn guard(&self, req_head: &RequestHead) -> bool {
        if req_head.headers.contains_key("accept") {
            if req_head
                .headers
                .get("accept")
                .expect("guarded")
                .to_str()
                .expect("ed")
                .contains("text/html")
            {
                return true;
            }
        }
        return false;
    }
}

#[derive(Debug, Clone)]
pub struct Script2;

impl RespMod for Script2 {
    fn process_str(&self, str: String) -> String {
        str.replace("here!", "there!")
    }
    fn guard(&self, req_head: &RequestHead) -> bool {
        (Script).guard(req_head)
    }
}
