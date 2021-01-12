use node_bindgen::derive::node_bindgen;

#[node_bindgen]
async fn hello<F: Fn(String)>(bs_json: String, cb: F) {
    bs3_core::json::from_json(bs_json).await;
    cb(String::from("all done"));
}
