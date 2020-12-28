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

pub async fn main(browser_sync: BrowserSync) -> anyhow::Result<()> {
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

#[test]
fn test_entire_app() {
    #[derive(Debug, PartialEq)]
    enum Msg {
        Stop,
        Stopped,
        Ping(String),
    }
    #[derive(Debug, PartialEq)]
    enum ServerMsg {
        Listening(String),
    }

    let (outer_s, outer_r) = crossbeam_channel::unbounded::<Msg>();
    let (outer_s2, _outer_r2) = (outer_s.clone(), outer_r.clone());
    let (inner_s, inner_r) = crossbeam_channel::unbounded::<Msg>();
    let (server_s, server_r) = crossbeam_channel::unbounded::<ServerMsg>();

    let _h1 = std::thread::spawn(move || {
        outer_s.send(Msg::Ping(String::from("1"))).expect("send");
        actix::run(async move {
            println!("actix running... listening for stop msg...");
            server_s
                .send(ServerMsg::Listening(String::from("0.0.0.0:8080")))
                .expect("can ping");
            match inner_r.iter().find(|m| *m == Msg::Stop) {
                Some(_) => {
                    actix::System::current().stop();
                    outer_s.send(Msg::Stopped).expect("stopped")
                }
                _ => println!("ignoring..."),
            }
        })
        .expect("actix end");
    });
    let _h2 = std::thread::spawn(move || {
        outer_s2.send(Msg::Ping(String::from("2"))).expect("send");

        let msg = server_r.recv().expect("listening address");
        println!("running at {:?}", msg);
        inner_s.send(Msg::Stop).expect("can send stop");

        // std::thread::sleep(std::time::Duration::from_secs(2));
        // println!("sending stop from test code...");
        // actix::run(async move {
        //     println!("hey!");
        //     println!("stoppping...!");
        //     s.send(Msg::Stop).expect("can send stop");
        // });
    });
    outer_r.iter().for_each(|m| match m {
        Msg::Stopped => println!("YAY!!!! - stopped"),
        _cmd => println!("msg={:?}", _cmd),
    });
    // std::thread::spawn(move || {
    //     actix::run(async move {
    //         println!("2 init");
    //         for m in r2.iter() {
    //             match m {
    //                 Msg::Stop => {
    //                     println!("got stop msg");
    //                     actix::System::current().stop()
    //                 },
    //                 Msg::Done => println!("ignoring done"),
    //             }
    //         }
    //     });
    //     s2.send(Msg::Done).expect("can send");
    //     println!("--> after");
    // });
    // for m in r.iter() {
    //     match m {
    //         Msg::Done => println!("DONE!"),
    //         Msg::Stop => println!("end...{:?}", m)
    //     }
    // };
}
