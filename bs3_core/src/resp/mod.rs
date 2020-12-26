use std::future::Future;

use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{
    body::ResponseBody,
    dev::{RequestHead, ResponseHead, ServiceRequest, ServiceResponse},
    web::{self, Bytes, BytesMut},
    Error,
};

use actix_web::dev::Body;
use actix_web::http::header::CONTENT_ENCODING;
use actix_web::http::{ContentEncoding, HeaderValue};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use futures::future::{ok, Ready};
use futures_util::StreamExt;
use std::io;
use std::io::{Read, Write};

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
    fn process_str(&self, input: String, indexes: &[usize]) -> String;
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

    fn process_str(&self, input: String, indexes: &[usize]) -> String {
        indexes.iter().fold(input, |acc, index| {
            let item = self.items.get(*index).expect("guarded");
            log::debug!("processing [{}] {}", index, item.name());
            item.process_str(acc)
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
        let _uri = req.uri().clone();
        let srv_v = self.service.call(req);

        Box::pin(async move {
            let res = srv_v.await;
            match res {
                Ok(res) => {
                    let mut res: ServiceResponse = res;
                    let req = res.request().clone();
                    let uri_string = req.uri().to_string();

                    let head = req.head();
                    let response = res.response();

                    //
                    // These are the transformed registered in config
                    //
                    let transforms = req
                        .app_data::<web::Data<RespModData>>()
                        .map(|t| t.get_ref());

                    //
                    // 'indexes' are the transforms that should be applied to the body.
                    // eg: if 'indexes' is [0, 1] -> this means 2 transforms will be applied to this response
                    //
                    let indexes: Vec<usize> = transforms
                        .map(|trans| trans.indexes(&head, &response.head()))
                        .unwrap_or_else(Vec::new);

                    log::debug!("indexes to process = {:?}", indexes);

                    //
                    // Early return if no-one wants to edit this response
                    //
                    if indexes.is_empty() {
                        return Ok(res);
                    }

                    let mut body = BytesMut::new();
                    let mut stream = res.take_body();

                    while let Some(chunk) = stream.next().await {
                        log::debug!("++ chunk from buffered response body");
                        body.extend_from_slice(&chunk?);
                    }

                    //
                    // From the "content-encoding" header, determine if the response
                    // requires de-coding before we can modify it
                    //
                    let encoding = res
                        .response()
                        .headers()
                        .get("content-encoding")
                        .and_then(|val| val.to_str().ok())
                        .map(ContentEncoding::from)
                        .unwrap_or(ContentEncoding::Identity);

                    log::debug!("handling encoding: {:?}", encoding);

                    //
                    // decode the bytes if we can
                    //
                    let decoded_bytes: Bytes = match encoding {
                        ContentEncoding::Gzip => {
                            log::trace!("decoding a buffered gzip response");
                            let decoded = decode_gzip(body.to_vec()).expect("decode");
                            Bytes::from(decoded)
                        }
                        _ => Bytes::from(body),
                    };

                    //
                    // Process each transform on the content
                    //
                    process_buffered_body(decoded_bytes, uri_string, transforms, &indexes)
                        //
                        // Whether or not to re-encode the response, based on whether the original was
                        //
                        .map(|processes_bytes| match encoding {
                            ContentEncoding::Gzip => {
                                let encoded =
                                    encode_gzip(processes_bytes.to_vec()).expect("gzip encode");
                                Bytes::from(encoded)
                            }
                            _ => processes_bytes,
                        })
                        //
                        // Now with either modified bytes or original, we can re-send them
                        //
                        .map(|output_bytes| {
                            res.map_body(|head, _body| {
                                head.headers_mut().insert(
                                    CONTENT_ENCODING,
                                    HeaderValue::from_str(encoding.as_str())
                                        .expect("creation of this header never fails"),
                                );
                                ResponseBody::Body(Body::Bytes(output_bytes))
                            })
                        })
                }
                Err(..) => todo!(),
            }
        })
    }
}

///
/// Process the entire buffered body in 1 go, this avoids trying to match over
/// chunked responses etc
///
fn process_buffered_body(
    bytes: Bytes,
    uri: String,
    transforms: Option<&RespModData>,
    indexes: &[usize],
) -> Result<Bytes, Error> {
    let to_process = std::str::from_utf8(&bytes);
    match to_process {
        Ok(str) => {
            let string = String::from(str);
            if !indexes.is_empty() {
                log::debug!("processing indexes {:?} for `{}`", indexes, uri);
                let next = transforms
                    .map(|trans| trans.process_str(string.clone(), indexes))
                    .unwrap_or_else(String::new);
                return Ok(Bytes::from(next));
            }
            log::debug!("NOT processing indexes {:?} for `{}`", indexes, uri);
            Ok(Bytes::from(string))
        }
        Err(e) => {
            eprintln!("error converting bytes {:?}", e);
            Ok(bytes)
        }
    }
}

fn decode_gzip(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut d = GzDecoder::new(&bytes[..]);
    let mut s = Vec::new();
    d.read_to_end(&mut s).unwrap();
    Ok(s)
}

fn encode_gzip(input: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut e = GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(&input[..]).unwrap();
    e.finish()
}
