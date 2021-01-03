use actix_web::{App, HttpRequest, HttpResponse, HttpServer};

use tokio::sync::broadcast::Sender;

use crate::browser_sync::BrowserSync;

use crate::server::{Server, Start};
use actix::{Actor, Addr};
use actix_web::http::StatusCode;

#[derive(Debug, Clone)]
pub enum BrowserSyncMsg {
    Listening { bs: BrowserSync },
}

#[derive(Debug)]
pub enum Final {
    Stopped,
    Errored(anyhow::Error),
}

pub async fn main(
    _browser_sync: BrowserSync,
    _recv: Option<Sender<BrowserSyncMsg>>,
) -> anyhow::Result<Addr<Server>> {
    let addr = (Server { spawn_handle: None }).start();
    let add2 = addr.clone();
    let add23 = addr.clone();
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let (tx2, rx2) = tokio::sync::oneshot::channel::<()>();
    actix_rt::spawn(async move {
        println!("creating 1");
        match addr
            .send(Start {
                bind: String::from("127.0.0.1:8080"),
            })
            .await
        {
            Ok(rx) => {
                println!("listening...");
                // rx;
                println!("listening done......");
                tx.send(());
            }
            Err(e) => eprintln!("err={:?}", e),
        };
    });
    actix_rt::spawn(async move {
        println!("creating 1");
        match add2
            .send(Start {
                bind: String::from("127.0.0.1:8081"),
            })
            .await
        {
            Ok(rx) => {
                println!("listening...");
                // rx;
                println!("listening done......");
                tx2.send(());
            }
            Err(e) => eprintln!("err={:?}", e),
        };
    });
    // actix_rt::spawn(async move {
    //     println!("creating 2");
    //     match add2.clone().send(Start).await {
    //         Ok(rx) => {
    //             println!("listening...");
    //             // rx;
    //             println!("listening done......");
    //             tx.send(());
    //         }
    //         Err(e) => eprintln!("err={:?}", e),
    //     };
    // });
    futures::future::select(rx, rx2).await;
    Ok(add23)
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
