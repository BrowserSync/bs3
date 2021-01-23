use async_graphql::{Context, Result as GqlResult};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{mpsc::Sender, Mutex};

pub struct MutationRoot;

#[async_graphql::Object]
impl MutationRoot {
    async fn stop(&self, ctx: &Context<'_>) -> GqlResult<queries::MutationResult> {
        let stop_sender = ctx.data_unchecked::<Arc<Mutex<Sender<()>>>>();

        // Access the stop message sender
        let mut m = stop_sender.lock().await;

        // Send the stop message
        m.send(())
            .await
            .map(|_| queries::MutationResult::Stopped)
            .map_err(|_e| MutationError::CouldNotStop.into())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MutationError {
    #[error("Could not stop the server")]
    CouldNotStop,
}

mod query_dsl {
    cynic::query_dsl!("./static/schema.graphql");
}

#[cynic::query_module(schema_path = "./static/schema.graphql", query_module = "query_dsl")]
mod queries {

    use super::query_dsl;

    #[derive(Debug, Clone, Eq, PartialEq, Copy, async_graphql::Enum, cynic::Enum)]
    #[cynic(graphql_type = "MutationResult")]
    pub enum MutationResult {
        Stopped,
    }

    #[derive(cynic::QueryFragment, Debug)]
    #[cynic(graphql_type = "MutationRoot")]
    pub struct Stop {
        stop: MutationResult,
    }
}

fn stop_mutation() -> impl Serialize {
    use cynic::MutationBuilder;
    queries::Stop::build(())
}

#[test]
fn stop() {
    let stop = stop_mutation();
    let str = serde_json::to_string(&stop).unwrap();
    println!("str = {}", str);
}
