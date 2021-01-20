#[actix_web::main]
async fn main() {
    env_logger::init();
    let fut = bs3_core::entry::from_args(std::env::args().skip(1));
    match fut.await {
        Ok(_) => {
            println!("[bs3 bin] all servers closed");
        }
        Err(e) => {
            eprintln!("e={}", e);
            std::process::exit(1);
        }
    }
}
