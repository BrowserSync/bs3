use actix::Actor;
use actix_web::{http::StatusCode, web, App, HttpResponse, HttpServer};
use bs3_files::served::{Register, Served, ServedAddr};
use bs3_files::Files;

use crate::{
    client::css::Css, client::script::Script, fs::FsWatcher, resp, resp::RespModData,
    ws::server::WsServer, ws::ws::ws_route,
};

use bytes::Bytes;
use futures::StreamExt;

use crate::fs::RegisterFs;
use crate::browser_sync::BrowserSync;
use crate::serve_static::{ServeStaticConfig, ServeStatic};

use std::sync::Arc;

pub async fn main(browser_sync: BrowserSync) -> std::io::Result<()> {

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
        let mut app = App::new()
            .data(ws_server.clone())
            .data(mods)
            .data(served_addr.clone())
            .wrap(resp::RespModMiddleware)
            .service(web::resource("/__bs3/ws/").to(ws_route))
            .service(web::resource("/chunked").to(chunked_response))
            .service(Files::new(
                "/__bs3/client",
                "/Users/shakyshane/Sites/bs3/bs3_client/dist",
            ));

        let index = browser_sync.config
            .index
            .as_ref()
            .map(|s| s.to_owned())
            .unwrap_or_else(|| String::from("index.html"));

        println!("{:?}", browser_sync.config.serve_static_config());
        for ss in &browser_sync.config.serve_static_config() {
            match ss {
                ServeStaticConfig::DirOnly(pb) => {
                    app = app.service(Files::new("/", &pb).index_file(&index));
                },
                ServeStaticConfig::Multi { routes, dir } => {
                    for route in routes {
                        if let Some(as_str) = route.to_str() {
                            app = app.service(Files::new(as_str, &dir).index_file(&index));
                        }
                    }
                }
            };
        }


        app
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

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
