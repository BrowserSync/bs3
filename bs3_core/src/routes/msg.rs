use actix_web::{web, HttpRequest, HttpResponse};
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum IncomingHttpMsg {
    Stop,
}

/// http handler, should this be elsewhere?
pub async fn incoming_msg(_item: web::Json<IncomingHttpMsg>, req: HttpRequest) -> HttpResponse {
    let transforms = req
        .app_data::<web::Data<Arc<Mutex<tokio::sync::mpsc::Sender<()>>>>>()
        .map(|t| t.get_ref());
    if let Some(sender) = transforms {
        let mut m = sender.lock().unwrap();
        match m.send(()).await {
            Ok(_) => { /* noop */ }
            Err(e) => eprintln!(
                "could not send stop message from incoming_msg handler, {}",
                e
            ),
        }
    }
    HttpResponse::Ok().body("OK") // <- send json response
}

#[test]
fn test_http_stop_msg() {
    let json = r#"  {"kind": "Stop"} "#;
    let expected = IncomingHttpMsg::Stop;
    assert_eq!(
        serde_json::from_str::<IncomingHttpMsg>(json).expect("test"),
        expected
    );
}
