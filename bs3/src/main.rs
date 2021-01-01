use bs3_core::browser_sync::BrowserSync;
use bs3_core::start;
use bs3_core::start::Final;
use std::process::exit;
use tokio::sync::{broadcast, oneshot};

#[actix_web::main]
async fn main() {
    env_logger::init();
    let browser_sync = BrowserSync::try_from_args(std::env::args().skip(1));
    match browser_sync {
        Ok(browser_sync) => {
            log::debug!("{:#?}", browser_sync);
            let (tx, mut rx) = broadcast::channel(100);
            let (stop_msg_sender, stop_msg_receiver) = oneshot::channel::<i32>();
            actix_rt::spawn(async move {
                match rx.recv().await {
                    Ok(msg) => println!("message={:?}", msg),
                    Err(err) => {
                        log::trace!("missed a message... {}", err);
                    }
                }
            });
            actix_rt::spawn(async move {
                let fut = start::main(browser_sync, Some(tx));
                let exit_code = match fut.await {
                    Ok(Final::Stopped) => {
                        log::trace!("closing wth final stopped message");
                        0
                    }
                    Ok(Final::Errored(e)) => {
                        log::trace!("closing wth final error message {:?}", e);
                        eprintln!("error: {:?}", e);
                        1
                    }
                    Err(err) => {
                        eprintln!("error: {:?}", err);
                        1
                    }
                };
                if let Err(e) = stop_msg_sender.send(exit_code) {
                    eprintln!("failed to send stop message {:?}", e);
                }
            });
            match stop_msg_receiver.await {
                Ok(exit_code) => exit(exit_code),
                Err(e) => {
                    eprintln!("error = {}", e);
                    exit(1);
                }
            }
        }
        Err(err) => {
            eprintln!("~~~error: {:?}", err);
            exit(1);
        }
    };
}
