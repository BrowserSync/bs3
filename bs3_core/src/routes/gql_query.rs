use crate::browser_sync::BrowserSync;
use async_graphql::Context;
use std::sync::{Arc, Mutex};

pub struct BrowserSyncGraphData {
    pub bs_instances: Arc<Mutex<Vec<BrowserSync>>>,
}

pub struct QueryRoot;

#[async_graphql::Object]
impl QueryRoot {
    async fn servers(&self, ctx: &Context<'_>) -> Vec<BrowserSync> {
        let data = ctx.data_unchecked::<BrowserSyncGraphData>();
        let items = data.bs_instances.lock().unwrap();
        items.iter().map(|bs| bs.clone()).collect()
    }
}
