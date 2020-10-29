#[actix_web::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let browser_sync = bs3_core::browser_sync::BrowserSync::from_args(args.into_iter());
    let fut = bs3_core::start::main(browser_sync);
    fut.await;
}
