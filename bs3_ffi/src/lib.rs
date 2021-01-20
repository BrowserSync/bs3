use node_bindgen::derive::node_bindgen;

#[node_bindgen]
async fn start<F: Fn(String)>(bs_json: String, cb: F) {
    match bs3_core::entry::from_json(bs_json).await {
        Ok(_) => {
            // println!("all good in fn start")
        }
        Err(e) => println!("err in fn start {:?}", e),
    };
    cb(String::from("all done"));
}

#[node_bindgen]
async fn stop<F: Fn(String)>(addr: String, cb: F) {
    match bs3_core::server::stop::stop(addr).await {
        Ok(_) => { /* noop */ }
        Err(_e) => {
            eprintln!("error from sending the stop message")
        }
    }
    cb(String::from("OK"));
}
