use bs3_core::browser_sync::BrowserSync;
use bs3_core::start;
use bs3_core::start::Final;
use std::process::exit;

#[actix_web::main]
async fn main() {
    env_logger::init();
    let browser_sync = BrowserSync::try_from_args(std::env::args().skip(1));
    dbg!(&browser_sync);
    match browser_sync {
        Ok(browser_sync) => {
            log::debug!("{:#?}", browser_sync);
            let fut = start::main(browser_sync, None);

            exit(match fut.await {
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
            });
        }
        Err(err) => {
            eprintln!("error: {:?}", err);
            exit(1);
        }
    };
}
