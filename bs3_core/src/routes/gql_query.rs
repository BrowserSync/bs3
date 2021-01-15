use crate::browser_sync::BrowserSync;
use crate::routes::gql_models::BrowserSyncServer;
use async_graphql::{Context, Enum, Object, Result, Schema, Subscription, ID};
use std::sync::{Arc, Mutex};

pub struct BrowserSyncGraphData {
    pub bs_instances: Arc<Mutex<Vec<BrowserSync>>>,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn servers(&self, ctx: &Context<'_>) -> Vec<BrowserSyncServer> {
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
