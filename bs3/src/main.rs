#[actix_web::main]
async fn main() {
    env_logger::init();
    let browser_sync = bs3_core::browser_sync::BrowserSync::try_from_args(std::env::args());
    match browser_sync {
        Ok(browser_sync) => {
            log::debug!("{:#?}", browser_sync);
            let fut = bs3_core::start::main(browser_sync, None);

            std::process::exit(match fut.await {
                Ok(_) => 0,
                Err(err) => {
                    eprintln!("error: {:?}", err);
                    1
                }
            });
        }
        Err(err) => {
            eprintln!("error: {:?}", err);
            std::process::exit(1);
        }
    };
}
