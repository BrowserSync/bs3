use crate::browser_sync::BrowserSync;
use crate::start;
use actix_rt::time::delay_for;
use actix_web::http::StatusCode;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};

pub fn from_json(json: String) -> anyhow::Result<()> {
    actix_rt::System::new("bs3_core::from_json").block_on(async move {
        println!("starting server...");
        println!("trying to construct a browser sync instance from {}", json);
        HttpServer::new(move || App::new().service(welcome))
            .disable_signals()
            .bind("0.0.0.0:8080")?
            .run()
            .await?;
        println!("after server...");
        Ok(())
    })
}

#[actix_web::get("/")]
async fn welcome(_req: HttpRequest) -> actix_web::Result<HttpResponse> {
    // response
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body("hello world"))
}
