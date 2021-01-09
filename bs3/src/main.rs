use bs3_core::browser_sync::BrowserSync;
use bs3_core::start;

use std::process::exit;

#[actix_web::main]
async fn main() {
    env_logger::init();
    let browser_sync = BrowserSync::try_from_args(std::env::args().skip(1));
    match browser_sync {
        Ok(browser_sync) => {
            log::debug!("{:#?}", browser_sync);
            let fut = start::main(browser_sync, None);
            match fut.await {
                Ok(_) => {
                    println!("all done");
                }
                Err(e) => {
                    eprintln!("e={}", e);
                }
            }
        }
        Err(err) => {
            eprintln!("~~~error: {:?}", err);
            exit(1);
        }
    };
}
