use crate::serve_static::ServeStaticConfig;
use actix_multi::service::{MultiServiceFuture, MultiServiceTrait};
use actix_service::Service;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, Error, HttpResponse};
use futures::future::{ready, Either};
use std::sync::Arc;
use std::task::{Context, Poll};

#[derive(Debug, Clone)]
pub struct NotFound;

impl MultiServiceTrait for NotFound {
    fn check_multi(&self, _req: &ServiceRequest) -> bool {
        true
    }
}

impl Service for NotFound {
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Future = MultiServiceFuture;

    fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let served = req
            .app_data::<web::Data<Arc<Vec<ServeStaticConfig>>>>()
            .map(|t| t.get_ref());

        // todo: output more stuff here
        let config = served
            .and_then(|served| {
                let cloned = &*served.clone();
                serde_json::to_string_pretty(&cloned).ok()
            })
            .unwrap_or(String::from("unknown"));

        let resp = HttpResponse::Ok()
            .body(include_str!("../../static/bs_404.html").replace("{config}", &config));
        let (req, _) = req.into_parts();
        let srv_resp = ServiceResponse::new(req, resp);
        Either::Left(ready(Ok(srv_resp)))
    }
}
