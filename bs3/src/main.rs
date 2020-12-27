#[actix_web::main]
async fn main() {
    env_logger::init();
    let browser_sync = bs3_core::browser_sync::BrowserSync::from_args(std::env::args());
    log::debug!("{:#?}", browser_sync);
    let fut = bs3_core::start::main(browser_sync);

    std::process::exit(match fut.await {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}
