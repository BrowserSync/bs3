use crate::browser_sync::BrowserSync;
use crate::config::get_available_port;
use crate::start::main;
use actix_web::client::{Client, ClientResponse};
use actix_web::dev::{Decompress, Payload};
use actix_web::error::PayloadError;
use actix_web::http::header::ACCEPT;
use actix_web::web::Bytes;
use anyhow::Context;
use std::future::Future;
use std::pin::Pin;

type TestError = Option<String>;
type TestOutput = Result<TestError, anyhow::Error>;
type ExecReturn = Pin<Box<dyn Future<Output = TestOutput>>>;
type Resp = ClientResponse<
    Decompress<
        Payload<Pin<Box<dyn futures::Stream<Item = std::result::Result<Bytes, PayloadError>>>>>,
    >,
>;

pub struct Runner {
    pub bs: BrowserSync,
    pub name: String,
}

///
/// Construct an e2e test from CLI arguments
///
impl Runner {
    pub fn from_cli_args(
        name: impl Into<String>,
        args: Vec<impl Into<String>>,
    ) -> Result<Self, anyhow::Error> {
        let args = args.into_iter().map(|i| i.into());
        let mut bs = BrowserSync::try_from_args(args).context("Could not parse from args")?;

        // use a random port for tests to prevent any overlap (eg: to let
        // tests run in parallel if possible
        let p = get_available_port().expect("can select open port");
        bs.set_port(p);

        Ok(Runner {
            bs,
            name: name.into(),
        })
    }
    pub fn test(&self, tester: impl Fn(url::Url) -> ExecReturn + 'static) -> anyhow::Result<()> {
        runner(self.name.clone(), self.bs.clone(), tester)?;
        Ok(())
    }
    pub async fn req(url: &url::Url, path: &str) -> Result<Resp, anyhow::Error> {
        let client = Client::default();
        let mut local_url = url.clone();
        local_url.set_path(path);
        Ok(client
            .get(local_url.to_string())
            .header(ACCEPT, "*/*")
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?)
    }
    pub fn assert_status(res: Resp, status: u16) -> TestOutput {
        if res.status() != status {
            return Ok(Some(format!(
                "expected status {}, got {}",
                status,
                res.status()
            )));
        }
        Ok(None)
    }
}

fn runner(
    name: String,
    bs: BrowserSync,
    tester: impl Fn(url::Url) -> ExecReturn + 'static,
) -> anyhow::Result<()> {
    #[derive(Debug, PartialEq)]
    enum ServerMsg {
        Listening(url::Url),
    }
    #[derive(Debug, PartialEq)]
    enum Status {
        Stopped,
        Error(String),
    }

    actix_rt::System::new("test-system").block_on(async move {
        let (mut tx, mut rx) = tokio::sync::mpsc::channel::<Status>(1);
        let (mut server_tx, mut server_rx) = tokio::sync::mpsc::channel::<ServerMsg>(1);
        actix_rt::spawn(async move {
            server_tx
                .send(ServerMsg::Listening(bs.local_url.0.clone()))
                .await
                .expect("can send listening message");
            match main(bs.clone(), None).await {
                Ok(_) => log::trace!("server closed cleanly"),
                Err(e) => log::error!("{}", e),
            };
        });
        actix_rt::spawn(async move {
            match server_rx.recv().await {
                Some(ServerMsg::Listening(url)) => {
                    match tester(url).await {
                        Ok(Some(error)) => {
                            tx.send(Status::Error(error)).await.expect("can send error")
                        }
                        _ => tx
                            .send(Status::Stopped)
                            .await
                            .expect("can send stopped message"),
                    };
                    actix_rt::System::current().stop();
                }
                _cmd => todo!("msg not supported"),
            };
        });
        match rx.recv().await {
            Some(Status::Error(error_str)) => {
                eprintln!("`{}` failed", name);
                eprintln!("error={}", error_str);
                panic!("{}", error_str);
            }
            Some(Status::Stopped) => println!("done!"),
            None => println!("none..."),
        }
    });
    Ok(())
}
