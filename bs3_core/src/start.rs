use actix::Actor;
use actix_web::{web, App, HttpServer};
use bs3_files::served::{Register, Served, ServedAddr};
use bs3_files::{Files, FilesService};

use crate::{
    client::css::Css, client::script::Script, fs::FsWatcher, resp, resp::RespModData,
    ws::server::WsServer, ws::ws_session::ws_route,
};

use crate::browser_sync::BrowserSync;
use crate::fs::RegisterFs;
use crate::serve_static::{ServeStatic, ServeStaticConfig};

use crate::proxy::Proxy;
use crate::routes::not_found::NotFound;
use actix_multi::service::MultiServiceTrait;
use std::sync::Arc;

use crate::proxy::proxy_resp_mod::ProxyResp;
use crate::proxy::service::ProxyService;
use actix_web::client::Client;
use wasm_bindgen::__rt::core::time::Duration;
use actix_rt::time::delay_for;

#[derive(Debug)]
pub enum BrowserSyncMsg {
    Stop,
}

pub async fn main(
    browser_sync: BrowserSync,
    recv: Option<crossbeam_channel::Receiver<BrowserSyncMsg>>,
) -> anyhow::Result<()> {
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

    let ss_config = browser_sync.config.serve_static_config();
    let ss_config_arc = Arc::new(ss_config);

    let proxy_config = browser_sync.config.proxies();
    let proxy_config_arc = Arc::new(proxy_config);

    let local_url = browser_sync.local_url.0.clone();
    let clone_url = Arc::new(local_url);
    let bind_address = browser_sync.bind_address();

    let server = HttpServer::new(move || {
        let ss_config_arc = ss_config_arc.clone();
        let proxy_config_arc = proxy_config_arc.clone();
        let local_url = clone_url.clone();

        let mut mods = RespModData {
            items: vec![Box::new(Script), Box::new(Css)],
        };

        // if the proxy is configured & has no path - assume the entire website is being proxied
        if !proxy_config_arc.is_empty() {
            let first_without_paths = proxy_config_arc.iter().find(|pt| pt.paths.is_empty());
            if let Some(first) = first_without_paths {
                log::debug!("adding a proxy resp for {:?}", first.target);
                mods.items.push(Box::new(ProxyResp {
                    target_url: first.target.clone(),
                    local_url: (*local_url).clone(),
                }))
            }
        }

        let mut app = App::new()
            .data(ws_server.clone())
            .data(Client::new())
            .data(mods)
            .data(served_addr.clone())
            .data(ss_config_arc.clone())
            .wrap(resp::RespModMiddleware)
            .service(web::resource("/__bs3/ws/").to(ws_route))
            .service(Files::new(
                "/__bs3/client",
                "/Users/shakyshane/Sites/bs3/bs3_client/dist",
            ));

        let index = browser_sync
            .config
            .index
            .as_ref()
            .map(|s| s.to_owned())
            .unwrap_or_else(|| String::from("index.html"));

        app = app.service(actix_multi::service::Multi::new(move || {
            // create the fallthrough services
            let mut multi_services: Vec<Box<dyn MultiServiceTrait>> = vec![];

            // the FS based services
            let fs_services: Vec<FilesService> =
                ss_config_arc.clone().iter().fold(vec![], |mut acc, item| {
                    match item {
                        ServeStaticConfig::DirOnly(dir) => {
                            acc.push(Files::new("/", dir).index_file(&index).to_service());
                        }
                        ServeStaticConfig::Multi(multi) => {
                            for r in &multi.routes {
                                acc.push(
                                    Files::new(&r, multi.dir.clone())
                                        .index_file(&index)
                                        .to_service(),
                                );
                            }
                        }
                    };
                    acc
                });

            for s in fs_services {
                multi_services.push(Box::new(s))
            }

            // add the proxy config if present
            proxy_config_arc.iter().for_each(|p| {
                multi_services.push(Box::new(ProxyService {
                    targets: vec![p.clone()],
                }))
            });

            // add the not Found page
            multi_services.push(Box::new(NotFound));

            multi_services
        }));

        app
    });

    server
        .workers(1)
        .bind(bind_address)?
        .run()
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::{ACCEPT, ACCEPT_ENCODING};
    use std::future::Future;
    use std::pin::Pin;
    use actix_web::http;

    type TestError = Option<String>;

    fn runner(args: Vec<&'static str>, tester: impl Fn(url::Url) -> Pin<Box<dyn Future<Output=Result<TestError, anyhow::Error>>>> + 'static) {
        println!("hey");
        #[derive(Debug, PartialEq)]
        enum ServerMsg {
            Listening(url::Url),
        }
        #[derive(Debug, PartialEq)]
        enum Status {
            Stopped,
            Error(String)
        }
        struct Stop;

        actix_rt::System::new("test90121").block_on(async move {
            let (mut tx, mut rx) = tokio::sync::mpsc::channel::<Status>(1);
            let (mut server_tx, mut server_rx) = tokio::sync::mpsc::channel::<ServerMsg>(1);
            actix_rt::spawn(async move {
                let bs = BrowserSync::try_from_args(args.into_iter()).expect("bs test");
                server_tx
                    .send(ServerMsg::Listening(bs.local_url.0.clone()))
                    .await;
                match main(bs, None).await {
                    Ok(_) => log::trace!("server closed cleanly"),
                    Err(e) => log::error!("{}", e)
                };
            });
            actix_rt::spawn(async move {
                match server_rx.recv().await {
                    Some(ServerMsg::Listening(url)) => {
                        // actix_rt::blocking()
                        let mut rt = actix_rt::Runtime::new().unwrap();
                        match tester(url).await {
                            Ok(Some(error)) => tx.send(Status::Error(error)).await,
                            _ => tx.send(Status::Stopped).await,
                        };
                        actix_rt::System::current().stop();
                    }
                    _cmd => todo!("msg not supported")
                };
            });
            match rx.recv().await {
                Some(Status::Error(error_str)) => {
                    eprintln!("error={}", error_str);
                    panic!("{}", error_str);
                },
                Some(Status::Stopped) => println!("done!"),
                None => println!("none..."),
            }
        });
    }

    #[test]
    fn test_entire_app() {
        runner(vec!["bs", "/Users/shakyshane/Sites/bs3/fixtures/src", "--port", "9000"], |url| {
            Box::pin(async move {
                let mut client = Client::default();
                let mut local_url = url.clone();
                local_url.set_path("/kittens");
                let response1 = client.get(local_url.to_string() )
                    .header(ACCEPT, "*/*")
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("{}", e))?;

                if response1.status() != 200 {
                    return Ok(Some(String::from("none-200 error")));
                }

                Ok(None)
            })
        });
    }
}

