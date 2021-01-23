#[cfg(not(feature = "print-gql-schema"))]
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

#[cfg(feature = "print-gql-schema")]
fn main() -> Result<(), std::io::Error> {
    let str = bs3_core::routes::gql::print();
    std::env::current_dir()
        .map(|pb| pb.join("bs3_core/static/schema.graphql"))
        .and_then(|abs| std::fs::write(abs, str))
}
