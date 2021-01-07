use tokio::sync::broadcast::Sender;

use crate::browser_sync::BrowserSync;

use crate::server::{Ping, Server, Start};
use actix::{Actor, Addr};
use actix_rt::time::delay_for;
use std::future::Future;
use std::pin::Pin;

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
    browser_sync: BrowserSync,
    _recv: Option<Sender<BrowserSyncMsg>>,
) -> anyhow::Result<Addr<Server>> {
    let addr = (Server { spawn_handle: None }).start();
    let addr2 = addr.clone();
    // to implement with https://docs.rs/futures/0.3.8/futures/stream/fn.select_all.html
    // actually, with https://docs.rs/futures/0.3.8/futures/stream/trait.StreamExt.html#method.for_each_concurrent
    // or https://docs.rs/futures/0.3.8/futures/future/fn.try_join_all.html
    let bs_items = vec![browser_sync];
    let to_futures = bs_items.iter().map(|bs_ref| {
        let addr = addr.clone();
        Box::pin(async move { addr.send(Start { bs: bs_ref.clone() }).await })
    });
    println!("About to watch");
    match futures::future::try_join_all(to_futures).await {
        Ok(vec) => println!("got the output {:?}", vec),
        Err(err) => println!("got the error {:?}", err),
    };
    println!("after");
    Ok(addr2)
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
