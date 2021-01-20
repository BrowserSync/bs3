use crate::browser_sync::BrowserSync;
use crate::routes::gql_query::BrowserSyncGraphData;
use async_graphql::{Context, Result as GqlResult};
use std::sync::Arc;

pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn stop(&self, ctx: &Context<'_>, port: u16) -> GqlResult<Vec<BrowserSync>> {
        let data = ctx.data_unchecked::<BrowserSyncGraphData>();
        {
            let items_1 = data.bs_instances.lock().unwrap();
            let _matched = items_1
                .iter()
                .find(|bs| bs.local_url.0.port().expect("must have a port") == port)
                .ok_or(MutationError::ServerNotFound)?;
        }
        // //
        // // get a match
        //
        // if let Err(e) = matched {}

        let stop_sender =
            ctx.data_unchecked::<Arc<tokio::sync::Mutex<tokio::sync::mpsc::Sender<()>>>>();
        let mut m = stop_sender.lock().await;
        match m.send(()).await {
            Ok(_) => { /* noop */ }
            Err(e) => eprintln!(
                "could not send stop message from incoming_msg handler, {}",
                e
            ),
        };
        let data = ctx.data_unchecked::<BrowserSyncGraphData>();
        let items = data.bs_instances.lock().unwrap();
        Ok(items
            .iter()
            .filter(|bs| bs.local_url.0.port().expect("must have a port") != port)
            .map(|bs| bs.clone())
            .collect())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MutationError {
    #[error("Server Not Found")]
    ServerNotFound,
}
