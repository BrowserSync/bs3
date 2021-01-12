use actix_rt::time::delay_for;

pub async fn from_json(json: String) {
    actix_rt::System::new("bs3_core::from_json").block_on(async move {
        println!("starting...");
        delay_for(std::time::Duration::from_secs(5)).await;
        println!("finished...");
    });
}
