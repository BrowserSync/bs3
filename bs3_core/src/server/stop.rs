use crate::routes::gql::GQL_ENDPOINT;
use crate::routes::gql_mutation::stop_mutation;

pub async fn stop(addr: String) -> Result<(), anyhow::Error> {
    actix_rt::System::new("bs3_core::stop::msg").block_on(async move {
        let mut url = url::Url::parse(addr.as_str())?;
        url.set_path(GQL_ENDPOINT);
        let client = actix_web::client::Client::new();

        // Create request builder, configure request and send
        let mut response = client
            .post(url.as_str())
            .header("User-Agent", "Actix-web")
            .send_json(&stop_mutation())
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;

        // read response body
        let _body = response.body().await?;

        Ok(())
    })
}
