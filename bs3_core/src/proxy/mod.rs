use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::error::{Error};
use actix_service::Service;
use std::task::{Context, Poll};
use actix_web::HttpResponse;
use std::str::FromStr;
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use actix_multi::service::MultiServiceFuture;
use futures::future::{Either, ok};
use serde::{Deserialize, Deserializer, de, Serialize, Serializer};

pub trait Proxy: Default {
    fn proxies(&self) -> Vec<ProxyTarget>;
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct ProxyTarget {
    #[serde(serialize_with = "serialize_proxy")]
    pub target: url::Url
}

fn serialize_proxy<S>(input: &url::Url, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let as_string = input.to_string();
    serializer.serialize_str(&as_string)
}

impl FromStr for ProxyTarget {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ProxyTarget {
            target: url::Url::parse(s)?
        })
    }
}

impl<'de> Deserialize<'de> for ProxyTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

#[test]
fn test_serialize() {
    let p = ProxyTarget::from_str("http://www.example.com").expect("test");
    let str = serde_json::to_string_pretty(&p).expect("json");
    dbg!(str);
}

#[derive(Debug, thiserror::Error)]
enum ProxyError {
    #[error("invalid target: {0}")]
    InvalidTarget(String)
}

impl actix_multi::service::MultiServiceTrait for ProxyTarget {
    fn check_multi(&self, req: &ServiceRequest) -> bool {
        true
    }
}

impl actix_service::Service for ProxyTarget {
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Future = MultiServiceFuture;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let (req, _) = req.into_parts();
        let res = ServiceResponse::new(req.clone(), HttpResponse::Ok().body("yep!"));
        Either::Right(
            ok(res).boxed_local()
        )
    }
}
