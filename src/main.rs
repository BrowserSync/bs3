// mod resp;
mod client;
mod resp;
// mod resp2;

use actix_files::Files;
use actix_web::{App, HttpServer};

use crate::client::script::{Script, Script2};
use crate::resp::RespModData;
// use crate::resp::Logging;
// use crate::resp::Logging;
// use crate::resp2::SayHi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");
    std::env::set_var("RUST_LOG", "bs3=debug");
    env_logger::init();

    HttpServer::new(|| {
        let mods = RespModData {
            items: vec![Box::new(Script), Box::new(Script2)],
        };
        App::new()
            // Enable the logger.
            // .wrap(middleware::Logger::default())
            // .wrap(SayHi)
            // .wrap(resp::Logging)
            .data(mods)
            .wrap(resp::RespModMiddleware)
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
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
