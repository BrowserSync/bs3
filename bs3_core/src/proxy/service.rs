use crate::proxy::ProxyTarget;
use actix_multi::service::MultiServiceFuture;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, Error, HttpResponse};
use futures::future::Either;
use std::task::{Context, Poll};

use actix_web::client::Client;

use actix_web::http::header::HOST;
use actix_web::http::HeaderValue;

pub struct ProxyService {
    pub targets: Vec<ProxyTarget>,
}

impl actix_multi::service::MultiServiceTrait for ProxyService {
    fn check_multi(&self, req: &ServiceRequest) -> bool {
        req.uri()
            .path_and_query()
            .map(|pq| {
                let path_str = pq.path();
                let matches_1 = self.targets.iter().any(|target| {
                    target.paths.iter().any(|path| {
                        path_str.starts_with(path.to_str().expect("pathbuf must convert to str"))
                    })
                });
                log::trace!("route=[{}], matches=[{}]", path_str, matches_1);
                matches_1
            })
            .unwrap_or(false)
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
        let (req, body) = req.into_parts();
        let target = self.targets.get(0).expect("at least 1 exists").clone();
        let target_host = target
            .target
            .host_str()
            .expect("must be able to access host")
            .to_string();
        log::trace!("target_host={}", target_host);
        log::trace!("proxying [{}] to {}", req.uri(), target.target);

        Either::Right(Box::pin(async move {
            let client = req
                .app_data::<web::Data<Client>>()
                .map(|t| t.get_ref())
                .expect("Client must exist");

            let mut next_uri = target.target.clone();

            // Set the path for the remote.
            //  - if the path for the target is "/" it means
            //    the user wants to forward requests as they are
            //    eg: "/gql" -> "/gql"
            //    However if the remote has a path, use that instead
            let next_path = match target.target.path() {
                "/" => req.uri().path(),
                path => path,
            };
            next_uri.set_path(next_path);
            next_uri.set_query(req.uri().query());

            log::trace!("next_uri = {:?}", next_uri);
            // log::trace!("next_head = {:#?}", req.head());

            // let forwarded = client.request_from(next_uri.as_str(), req.head());
            let mut forwarded = client
                .request_from(next_uri.as_str(), req.head())
                .no_decompress();

            forwarded.headers_mut().insert(
                HOST,
                HeaderValue::from_str(target_host.as_str()).expect("unwrap"),
            );
            // forwarded.headers_mut().insert(ACCEPT_ENCODING, HeaderValue::from_str("identity").expect("unwrap"));
            forwarded.headers_mut().remove("upgrade-insecure-requests");

            log::trace!("forwarding... {:?}", forwarded);

            let mut res = forwarded.send_stream(body).await?;
            // let mut res = forwarded_req.send_body(body).await.map_err(Error::from)?;

            log::trace!("sent body stream");
            log::trace!("res = {:?}", res);

            let mut client_resp = HttpResponse::build(res.status());

            for (header_name, header_value) in
                res.headers().iter().filter(|(h, _)| *h != "connection")
            {
                log::trace!(
                    "setting header {:?}={:?}",
                    header_name.clone(),
                    header_value.clone()
                );
                client_resp.header(header_name.clone(), header_value.clone());
            }

            let body = res.body().await?;
            let res1 = client_resp.body(body);
            let res = ServiceResponse::new(req.clone(), res1);
            Ok(res)
        }))
    }
}

#[actix_web::main]
#[test]
async fn main_test() -> Result<(), Error> {
    std::env::set_var("RUST_LOG", "actix_http=trace");
    env_logger::init();

    let client = Client::new();

    // Create request builder, configure request and send
    let mut response = client
        .get("http://www.example.com")
        .header("User-Agent", "Actix-web")
        .send()
        .await?;

    // server http response
    println!("Response: {:?}", response);

    // read response body
    let body = response.body().await?;
    println!("Downloaded: {:?} bytes", body.len());

    Ok(())
}
