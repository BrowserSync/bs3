#[actix_web::main]
async fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let fut = bs3_core::start::main(args.into_iter());
    fut.await;
}
