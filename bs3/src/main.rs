// mod resp;
mod client;
mod fs;
mod resp;
mod ws;
// mod resp2;

use actix::Actor;
use actix_web::{http::StatusCode, web, App, HttpResponse, HttpServer};
use bs3_files::served::{Register, Served, ServedAddr};
use bs3_files::Files;

use crate::{
    client::css::Css, client::script::Script, fs::FsWatcher, resp::RespModData,
    ws::server::WsServer, ws::ws_route,
};

use bytes::Bytes;
use futures::StreamExt;

use crate::fs::RegisterFs;
use std::sync::Arc;
// use crate::resp::Logging;
// use crate::resp::Logging;
// use crate::resp2::SayHi;

async fn chunked_response() -> HttpResponse {
    let bytes = vec![
        "<!doctype html>
        <html lang='en'>
        <head>",
        "<meta charset='UTF-8'>
        <meta name='viewport' content='width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0'>
        <meta http-equiv='X-UA-Compatible' content='ie=edge'>
        <title>Document</title>
        </head>
        <body>
          <h1>Chunked</h1>
          <script src='app.js'></script>
        </body>
        </html>"
    ];
    let stream = futures::stream::iter(bytes).map(|str| Ok(Bytes::from(str)) as Result<Bytes, ()>);
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .streaming(stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "actix_web=info,bs3=debug,trace");
    // std::env::set_var("RUST_LOG", "bs3=debug,trace");
    env_logger::init();

    let ws_server = WsServer::default().start();
    let fs_server = FsWatcher::default().start();

    // service for tracking served static files
    let served = Served::default().start();

    let served_addr = Arc::new(ServedAddr(served.clone()));

    // let the FS watcher know when a file is served from disk
    served.do_send(Register {
        addr: fs_server.clone().recipient(),
    });

    fs_server.do_send(RegisterFs {
        addr: ws_server.clone().recipient(),
    });

    HttpServer::new(move || {
        let mods = RespModData {
            items: vec![Box::new(Script), Box::new(Css)],
        };
        App::new()
            .data(ws_server.clone())
            .data(mods)
            .data(served_addr.clone())
            .wrap(resp::RespModMiddleware)
            .service(web::resource("/__bs3/ws/").to(ws_route))
            .service(web::resource("/chunked").to(chunked_response))
            .service(Files::new("/", "./fixtures").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
