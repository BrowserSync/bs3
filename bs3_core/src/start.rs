use actix::Actor;
use actix_web::{client::Client, web, App, HttpServer};
use std::sync::Arc;

use bs3_files::{
    served::{Register, Served, ServedAddr},
    Files, FilesService,
};

use actix_multi::service::MultiServiceTrait;
use tokio::sync::broadcast::Receiver;
use tokio::sync::oneshot;

use crate::{
    browser_sync::BrowserSync,
    bs_error::BsError,
    client::css::Css,
    client::script::Script,
    fs::FsWatcher,
    fs::RegisterFs,
    proxy::proxy_resp_mod::ProxyResp,
    proxy::service::ProxyService,
    proxy::Proxy,
    resp,
    resp::RespModData,
    routes::not_found::NotFound,
    serve_static::{ServeStatic, ServeStaticConfig},
    ws::server::WsServer,
    ws::ws_session::ws_route,
};

#[derive(Debug)]
pub enum BrowserSyncMsg {
    Listening { bs: BrowserSync },
}

#[derive(Debug)]
pub enum Final {
    Stopped,
    Errored(anyhow::Error),
}

pub async fn main(
    browser_sync: BrowserSync,
    _recv: Option<Receiver<BrowserSyncMsg>>,
) -> anyhow::Result<Final> {
    let ws_server = WsServer::default().start();
    let fs_server = FsWatcher::default().start();
    let (stop_msg_sender, stop_msg_receiver) = oneshot::channel();

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
    let _target_port = local_url.port().expect("must have a port set here");
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

    actix_rt::spawn(async move {
        let binded = server.workers(1).bind(bind_address);
        if let Err(e) = binded {
            stop_msg_sender
                .send(Final::Errored(anyhow::anyhow!(e)))
                .expect("can send inner stop message");
        } else {
            let running: Result<_, anyhow::Error> = binded.unwrap().run().await.map_err(|e| {
                BsError::Unknown {
                    e: anyhow::anyhow!(e),
                }
                .into()
            });
            match running {
                Ok(_) => stop_msg_sender
                    .send(Final::Stopped)
                    .expect("can send inner stop message"),
                Err(e) => {
                    eprintln!("An error occurred {}", e);
                }
            }
        }
    });
    stop_msg_receiver.await.map_err(|e| {
        BsError::Unknown {
            e: anyhow::anyhow!(e),
        }
        .into()
    })
}

#[cfg(test)]
mod tests {
    use crate::test_utils::Runner;

    fn dir(path: &str) -> String {
        let mut cwd = std::env::current_dir().expect("current_dir");
        if cwd.ends_with("bs3_core") {
            cwd.pop();
        }
        cwd.join(path).to_string_lossy().to_string()
    }

    #[test]
    fn test_200() -> anyhow::Result<()> {
        let name = "testing homepage gives 200 when a valid path is given";
        let dir = dir("fixtures/src");
        let dir = vec![dir.as_str()];
        Runner::from_cli_args(name, dir)?.test(|url| {
            Box::pin(async move { Runner::assert_status(Runner::req(&url, "/").await?, 200) })
        })
    }
    #[test]
    fn test_200_ss() -> anyhow::Result<()> {
        let name = "testing homepage gives 200 when given with --serve-static flag";
        let dir = dir("fixtures/src");
        let args = vec!["--serve-static", dir.as_str()];
        Runner::from_cli_args(name, args)?.test(|url| {
            Box::pin(async move { Runner::assert_status(Runner::req(&url, "/").await?, 200) })
        })
    }
    #[test]
    fn test_404() -> anyhow::Result<()> {
        let name = "Testing a 404 response is given when no static files or proxy given";
        let args: Vec<&str> = vec![];
        Runner::from_cli_args(name, args)?.test(|url: url::Url| {
            Box::pin(async move { Runner::assert_status(Runner::req(&url, "/").await?, 404) })
        })
    }
}
