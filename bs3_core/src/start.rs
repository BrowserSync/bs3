use tokio::sync::broadcast::Sender;

use crate::browser_sync::BrowserSync;

use crate::output::msg::BrowserSyncOutputMsg;
use crate::output::StdOut;
use crate::server::{Server, Start};
use actix::Actor;

#[derive(Debug)]
pub enum Final {
    Stopped,
    Errored(anyhow::Error),
}

pub async fn main(
    browser_sync: BrowserSync,
    _recv: Option<Sender<BrowserSyncOutputMsg>>,
) -> anyhow::Result<()> {
    let std_output = StdOut::default().start();
    let addr = Server::default().start();
    let _bs_default = BrowserSync::from_random_port();
    let bs_items = vec![browser_sync];

    let to_futures = bs_items.iter().map(move |bs_ref| {
        let std_output = std_output.clone();
        addr.send(Start {
            bs: bs_ref.clone(),
            output_recipients: Some(vec![std_output.recipient()]),
        })
    });

    futures::future::try_join_all(to_futures)
        .await
        .map_err(|e| anyhow::anyhow!(e))
        .map(|_items| ())
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
