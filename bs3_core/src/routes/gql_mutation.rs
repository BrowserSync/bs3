use async_graphql::{Context, Result as GqlResult};
use std::sync::Arc;

pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn stop(&self, ctx: &Context<'_>) -> GqlResult<MutationResult> {
        let stop_sender =
            ctx.data_unchecked::<Arc<tokio::sync::Mutex<tokio::sync::mpsc::Sender<()>>>>();

        // Access the stop message sender
        let mut m = stop_sender.lock().await;

        // Send the stop message
        m.send(())
            .await
            .map(|_| MutationResult::Stopped)
            .map_err(|_e| MutationError::CouldNotStop.into())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MutationError {
    #[error("Could not stop the server")]
    CouldNotStop,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy, async_graphql::Enum)]
pub enum MutationResult {
    Stopped,
}
