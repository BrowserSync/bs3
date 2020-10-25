// mod resp;
mod client;
mod resp;
mod ws;
mod fs;
// mod resp2;

use actix::Actor;
use bs3_files::{Files};
use bs3_files::served::{ServedFiles};
use actix_web::{
    web,
    App,
    HttpResponse,
    HttpServer,
    HttpRequest,
    HttpMessage,
    http::StatusCode,
    dev::ServiceResponse
};

use crate::{
    ws::chat_route,
    resp::RespModData,
    client::script::Script,
    client::css::Css,
    ws::server::{ChatServer, Message},
    fs::{FsWatcher, AddWatcher},
};

use bytes::Bytes;
use futures::StreamExt;
use futures::future::{ok, Either};
use std::path::PathBuf;
use actix_service::{Service, IntoServiceFactory, ServiceFactory};
use std::sync::{Mutex, Arc};
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
    let ws_server = ChatServer::default().start();
    let fs_server = FsWatcher::default().start();
    let served = Arc::new(Mutex::new(ServedFiles::default()));

    fs_server.do_send(AddWatcher { pattern: std::path::PathBuf::from("./fixtures") });

    HttpServer::new(move || {
        let mods = RespModData {
            items: vec![Box::new(Script), Box::new(Css)],
        };
        App::new()
            // Enable the logger.
            // .wrap(middleware::Logger::default())
            // .wrap(SayHi)
            // .wrap(resp::Logging)
            // .service(web::resource("/ws/").to(chat_route))
            .data(ws_server.clone())
            .data(mods)
            .data(served.clone())
            .wrap(resp::RespModMiddleware)
            .service(web::resource("/__bs3/ws/").to(chat_route))
            .service(web::resource("/chunked").to(chunked_response))
            // .wrap(resp2::Logging)
            // .wrap(Logging)
            // .wrap_fn(|req, srv| {
            //     let pathname = String::from(req.path());
            //     let query = String::from(req.query_string());
            //     let fut = srv.call(req);
            //     async move {
            //         let mut res: ServiceResponse<_> = fut.await?;
            //         let mut body = res.take_body();
            //         let mut bytes = BytesMut::new();
            //
            //         while let Some(item) = body.next().await {
            //             bytes.extend_from_slice(&item.unwrap());
            //         }
            //         let as_utf8 = bytes.to_vec();
            //         let as_string = std::str::from_utf8(&as_utf8).expect("utf8");
            //         println!("File contents ={}", as_string);
            //
            //
            //         Ok(res.map_body(move |head, body| {
            //             println!("{:?}", bytes);
            //             ResponseBody::Body(bytes)
            //         }))
            //     }
            // })
            // We allow the visitor to see an index of the images at `/images`.
            // .service(Files::new("/images", "static/images/").show_files_listing())
            // Serve a tree of static files at the web root and specify the index file.
            // Note that the root path should always be defined as the last item. The paths are
            // resolved in the order they are defined. If this would be placed before the `/images`
            // path then the service for the static images would never be reached.
            .service(Files::new("/", "./fixtures").index_file("index.html"))
            // .default_service(web::get().to(|req: HttpRequest| {
            //     let dir = PathBuf::from("/Users/shakyshane/sites/bs3/fixtures");
            //     // let real_path: PathBuf = match req.match_info().path().parse() {
            //     //     Ok(item) => item,
            //     //     Err(e) => todo!(),
            //     // };
            //     let real_path = PathBuf::from("styles.css");
            //     println!("real_path={:?}", real_path);
            //     println!("dir={:?}", dir);
            //     // full file path
            //     let path = match std::path::PathBuf::from(dir).join(&real_path).canonicalize() {
            //         Ok(path) => path,
            //         Err(e) => {
            //             eprintln!("{:?}", e);
            //             todo!()
            //         }
            //     };
            //     println!("joined={:?}", path);
            //     log::debug!("path={}", path.display());
            //     match NamedFile::open(path) {
            //         Ok(mut named_file) => {
            //             match named_file.into_response(&req) {
            //                 Ok(item) => {
            //                     item
            //                     // todo!();
            //                 }
            //                 Err(e) => {
            //                     // Either::Left(ok(ServiceResponse::from_err(e, req)))
            //                     todo!();
            //                 },
            //             }
            //         }
            //         Err(e) => todo!(),
            //     }
            // }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
