use crate::proxy::ProxyTarget;
use actix_multi::service::MultiServiceFuture;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpResponse};
use futures::future::{ok, Either};
use futures_util::FutureExt;
use std::task::{Context, Poll};

pub struct ProxyService {
    pub targets: Vec<ProxyTarget>,
}

impl actix_multi::service::MultiServiceTrait for ProxyService {
    fn check_multi(&self, _req: &ServiceRequest) -> bool {
        true
    }
}

impl actix_service::Service for ProxyService {
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Future = MultiServiceFuture;

    fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let (req, _) = req.into_parts();
        let res = ServiceResponse::new(req.clone(), HttpResponse::NotFound().finish());
        Either::Right(ok(res).boxed_local())
    }
}
