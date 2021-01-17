use crate::routes::gql_mutation::MutationRoot;
use crate::routes::gql_query::QueryRoot;
use actix_web::{web, HttpResponse};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptySubscription, Schema};
use async_graphql_actix_web::{Request, Response};

pub type BrowserSyncSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
pub async fn gql_response(schema: web::Data<BrowserSyncSchema>, req: Request) -> Response {
    schema.execute(req.into_inner()).await.into()
}

pub async fn gql_playgound() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/__bs/graphql").subscription_endpoint("/__bs/graphql"),
        ))
}
