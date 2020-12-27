use crate::service::{MultiServiceFuture, MultiServiceTrait};
use actix_service::Service;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpResponse};
use futures::future::{ready, Either};
use std::path::PathBuf;
use std::task::{Context, Poll};

#[derive(Debug, Clone)]
pub struct Files {
    pub route: PathBuf,
    pub dir: PathBuf,
}

impl Files {
    pub fn new(route: impl Into<PathBuf>, dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            route: route.into(),
        }
    }
    pub fn boxed(route: impl Into<PathBuf>, dir: impl Into<PathBuf>) -> Box<Self> {
        Box::new(Self::new(route, dir))
    }
}

impl MultiServiceTrait for Files {
    fn check_multi(&self, req: &ServiceRequest) -> bool {
        let q = req.uri().path_and_query();

        // can this handle the req?
        let handle = if let Some(p) = q {
            self.route.starts_with(p.path())
        } else {
            false
        };

        handle
    }
}

impl Service for Files {
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Future = MultiServiceFuture;

    fn poll_ready(&mut self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let resp =
            HttpResponse::Ok().body(format!("{}:{}", self.route.display(), self.dir.display()));
        let (req, _) = req.into_parts();
        let srv_resp = ServiceResponse::new(req, resp);
        Either::Left(ready(Ok(srv_resp)))
    }
}
