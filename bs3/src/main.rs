fn main() {
    std::process::exit(match bs3_core::start::main() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}
