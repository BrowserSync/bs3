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
        items.iter().cloned().collect()
    }
    async fn server_by_port(&self, ctx: &Context<'_>, port: u16) -> Vec<BrowserSync> {
        println!("port={}", port);
        let data = ctx.data_unchecked::<BrowserSyncGraphData>();
        let items = data.bs_instances.lock().unwrap();
        items
            .iter()
            .filter(|bs| bs.local_url.inner.port().expect("must have a port") == port)
            .cloned()
            .collect()
    }
}
