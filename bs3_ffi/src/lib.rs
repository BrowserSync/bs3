use std::ptr;

use node_bindgen::core::NjError;
use node_bindgen::derive::node_bindgen;

#[node_bindgen]
async fn hello<F: Fn(String)>(bs_json: String, cb: F) {
    match bs3_core::json::from_json(bs_json) {
        Ok(_) => println!("here... all good"),
        Err(e) => eprintln!("error"),
    };
    println!("after...");
    cb(String::from("done..."));
}
