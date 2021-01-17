use crate::routes::gql_models::BrowserSyncServer;
use crate::routes::gql_query::BrowserSyncGraphData;
use async_graphql::Context;

pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn stop(&self, ctx: &Context<'_>) -> Vec<BrowserSyncServer> {
        let data = ctx.data_unchecked::<BrowserSyncGraphData>();
        let items = data.bs_instances.lock().unwrap();
        items
            .iter()
            .map(|bs| BrowserSyncServer {
                addr: bs.bind_address(),
            })
            .collect()
    }
}
