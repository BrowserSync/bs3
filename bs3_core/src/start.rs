use actix::Actor;
use actix_web::{http::StatusCode, web, App, HttpResponse, HttpServer};
use bs3_files::served::{Register, Served, ServedAddr};
use bs3_files::{Files, FilesService};

use crate::{
    client::css::Css, client::script::Script, fs::FsWatcher, resp, resp::RespModData,
    ws::server::WsServer, ws::ws_session::ws_route,
};

use bytes::Bytes;
use futures::StreamExt;

use crate::browser_sync::BrowserSync;
use crate::fs::RegisterFs;
use crate::serve_static::{ServeStatic, ServeStaticConfig};

use crate::proxy::Proxy;
use crate::routes::not_found::NotFound;
use actix_multi::service::MultiServiceTrait;
use std::sync::Arc;

use crate::config::default_port;
use crate::proxy::proxy_resp_mod::ProxyResp;
use crate::proxy::service::ProxyService;
use actix_web::client::Client;
use actix_web::dev::Body;
use actix_service::ServiceFactory;

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

    let mut local_url = url::Url::parse("http://127.0.0.1:80").expect("hard coded");
    let port = browser_sync.config.port.or_else(default_port);
    if let Some(port) = port {
        log::trace!("setting port {}", port);
        local_url
            .set_port(Some(port))
            .map_err(|_e| anyhow::anyhow!("Could not set the port!"))?;
    }

    let clone_url = Arc::new(local_url.clone());
    let as_bind_address = format!(
        "{}:{}",
        local_url.host_str().expect("can't fail"),
        local_url.port().unwrap_or(80)
    );

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
        .bind(as_bind_address)?
        .run()
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

#[test]
fn test_entire_app() {
    enum Msg {
        Done
    }
    let (s, r) = crossbeam_channel::unbounded::<Msg>();
    let (s2, r2) = (s.clone(), r.clone());

    std::thread::spawn(|| {
        actix::run(async move {
            println!("1 init");
            std::thread::sleep(std::time::Duration::from_secs(1));
            println!("hey!");
        });
    });
    std::thread::spawn(|| {
        actix::run(async move {
            println!("2 init");
            std::thread::sleep(std::time::Duration::from_secs(2));
            s.send(Msg::Done).expect("can send");
        });
    });
    match r.recv() {
        Ok(Msg::Done) => println!("msg"),
        // Ok(_) => unimplemented!(),
        Err(e) => eprintln!("ERR ={}", e)
    };
}
