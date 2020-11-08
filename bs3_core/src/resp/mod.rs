use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{body::{BodySize, MessageBody, ResponseBody}, dev::{RequestHead, ResponseHead, ServiceRequest, ServiceResponse}, web::{self, Bytes, BytesMut}, Error, HttpRequest, HttpResponse};
use bytes::Buf;
use futures::future::{ok, Ready};
use actix_web::dev::Decompress;
use actix_web::http::ContentEncoding;
use actix::FinishStream;
use flate2::read::GzDecoder;
use std::io;
use std::io::{Write, Read};
use futures_util::StreamExt;
use flate2::write::ZlibEncoder;
use flate2::Compression;

///
/// Response Modifications
///
/// Allow easy string-manipulation of text-based HTTP
/// Response bodies
///
pub trait RespMod {
    ///
    /// Name to be used in debug/display situations
    ///
    fn name(&self) -> String;
    ///
    /// Gives access to the ENTIRE buffered response body
    ///
    fn process_str(&self, resp: String) -> String {
        resp
    }
    ///
    /// To prevent buffering/modifications on all requests,
    /// you need to implement this guard
    ///
    fn guard(&self, req_head: &RequestHead, res_head: &ResponseHead) -> bool;
}

pub trait RespModDataTrait {
    fn indexes(&self, req_head: &RequestHead, res_head: &ResponseHead) -> Vec<usize>;
    fn process_str(&self, input: String, indexes: &Vec<usize>) -> String;
}

pub struct RespModData {
    pub items: Vec<Box<dyn RespMod>>,
}

impl RespModDataTrait for RespModData {
    fn indexes(&self, req_head: &RequestHead, res_head: &ResponseHead) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                if item.guard(&req_head, &res_head) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect()
    }

    fn process_str(&self, input: String, indexes: &Vec<usize>) -> String {
        indexes.iter().fold(input, |acc, index| {
            let item = self.items.get(*index).expect("guarded");
            log::debug!("processing [{}] {}", index, item.name());
            return item.process_str(acc);
        })
    }
}

pub struct RespModMiddleware;

impl<S: 'static> Transform<S> for RespModMiddleware
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse, Error = Error>,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = LoggingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggingMiddleware { service })
    }
}

pub struct LoggingMiddleware<S> {
    service: S,
}

impl<'a, S> Service for LoggingMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<ServiceResponse, Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let uri = req.uri().clone();
        let srv_v = self.service.call(req);

        Box::pin(async move {
            let res = srv_v.await;
            match res {
                Ok(mut res) => {
                    let res: ServiceResponse = res;
                    let req = res.request().clone();

                    log::trace!("map_body for {}", req.uri().to_string());

                    let head = req.head();
                    let response = res.response();

                    let transforms = req
                        .app_data::<web::Data<RespModData>>()
                        .map(|t| t.get_ref());

                    let indexes: Vec<usize> = transforms
                        .map(|trans| trans.indexes(&head, &response.head()))
                        .unwrap_or(vec![]);

                    log::debug!("indexes to process = {:?}", indexes);

                    // if !indexes.is_empty() {
                    //     let mut body = BytesMut::new();
                    //     let mut stream = res.take_body();
                    //
                    //     while let Some(chunk) = stream.next().await {
                    //         body.extend_from_slice(&chunk?);
                    //     }
                    //
                    //     let decoded = decode_gzip(body.to_vec()).expect("decode");
                    //     println!("body decoded = |{:?}|", decoded);
                    //     let encoded = encode_gzip(decoded.into_bytes()).expect("encode");
                    //     println!("body encoded = |{:?}|", encoded);
                    //
                    //
                    //     // let req = res.request().clone();
                    //     Ok(res.map_body(|| {
                    //
                    //     }))
                    // } else {
                        // let req = res.request().clone();
                    // }
                    Ok(res)
                }
                Err(..) => todo!()
            }
        })
    }
}

fn process(
    bytes: Bytes,
    uri: String,
    transforms: Option<&RespModData>,
    indexes: &Vec<usize>,
) -> Poll<Option<Result<Bytes, Error>>> {
    let to_process = std::str::from_utf8(&bytes);
    match to_process {
        Ok(str) => {
            let string = String::from(str);
            if !indexes.is_empty() {
                log::debug!("processing indexes {:?} for `{}`", indexes, uri);
                let next = transforms
                    .map(|trans| trans.process_str(string.clone(), indexes))
                    .unwrap_or(String::new());
                return Poll::Ready(Some(Ok(Bytes::from(next))));
            }
            log::debug!("NOT processing indexes {:?} for `{}`", indexes, uri);
            Poll::Ready(Some(Ok(Bytes::from(string))))
        }
        Err(e) => {
            eprintln!("error converting bytes {:?}", e);
            Poll::Ready(Some(Ok(bytes)))
        }
    }
}

fn decode_gzip(bytes: Vec<u8>) -> io::Result<String> {
    let mut d = GzDecoder::new(&bytes[..]);
    let mut s = String::new();
    d.read_to_string(&mut s).unwrap();
    Ok(s)
}

fn encode_gzip(input: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&input[..]);
    e.finish()
}
