use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::{
    body::{BodySize, MessageBody, ResponseBody},
    dev::{RequestHead, ResponseHead, ServiceRequest, ServiceResponse},
    web::{self, Bytes, BytesMut},
    Error, HttpRequest,
};
use bytes::Buf;
use futures::future::{ok, Ready};

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

impl<S: 'static, B> Transform<S> for RespModMiddleware
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody + 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<BodyLogger<B>>;
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

impl<'a, S, B> Service for LoggingMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<BodyLogger<B>>;
    type Error = Error;
    type Future = WrapperStream<S, B>;

    fn poll_ready(&mut self, cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        WrapperStream {
            fut: self.service.call(req),
            _t: PhantomData,
        }
    }
}

#[pin_project::pin_project]
pub struct WrapperStream<S, B>
where
    B: MessageBody,
    S: Service,
{
    #[pin]
    fut: S::Future,
    _t: PhantomData<(B,)>,
}

impl<S, B> Future for WrapperStream<S, B>
where
    B: MessageBody,
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = Result<ServiceResponse<BodyLogger<B>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res: Result<ServiceResponse<_>, _> = futures::ready!(self.project().fut.poll(cx));

        Poll::Ready(res.map(|res| {
            let req = res.request().clone();
            res.map_body(move |_head, body| {
                log::trace!("map_body for {}", req.uri().to_string());
                let head = req.head();
                let transforms = req
                    .app_data::<web::Data<RespModData>>()
                    .map(|t| t.get_ref());
                let indexes: Vec<usize> = transforms
                    .map(|trans| trans.indexes(&head, &_head))
                    .unwrap_or(vec![]);
                ResponseBody::Body(BodyLogger {
                    body,
                    body_accum: BytesMut::new(),
                    process: !indexes.is_empty(),
                    indexes,
                    req,
                    eof: false,
                })
            })
        }))
    }
}

#[pin_project::pin_project]
pub struct BodyLogger<B> {
    #[pin]
    body: ResponseBody<B>,
    body_accum: BytesMut,
    process: bool,
    indexes: Vec<usize>,
    req: HttpRequest,
    eof: bool,
}

impl<B: MessageBody> MessageBody for BodyLogger<B> {
    fn size(&self) -> BodySize {
        if self.process {
            BodySize::Stream
        } else {
            self.body.size()
        }
    }

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>> {
        let mut this = self.project();
        let req: &mut HttpRequest = this.req;

        if is_ws_req(&req.head()) {
            return this.body.poll_next(cx);
        }

        let original_body_size = this.body.size().clone();
        let is_stream = original_body_size == BodySize::Stream;

        loop {
            let s = this.body.as_mut();
            return match s.poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    log::trace!("chunk size = {:?}", chunk.size());
                    if !*this.process {
                        log::trace!("chunk pass-thru");
                        return Poll::Ready(Some(Ok(chunk)));
                    }
                    this.body_accum.extend_from_slice(&chunk);
                    log::trace!(
                        "this.body_accum = {:?}, this.body = {:?}",
                        this.body_accum.size(),
                        original_body_size
                    );
                    if this.body_accum.size() == original_body_size {
                        let uri = req.uri().to_string();
                        let transforms = this
                            .req
                            .app_data::<web::Data<RespModData>>()
                            .map(|t| t.get_ref());
                        process(this.body_accum.to_bytes(), uri, transforms, &this.indexes)
                    } else {
                        if is_stream {
                            log::trace!("continue since this is a stream");
                            continue;
                        } else {
                            Poll::Pending
                        }
                    }
                }
                Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => {
                    if *this.eof {
                        log::trace!("early exit since this.eof = true");
                        return Poll::Ready(None);
                    }
                    if is_stream {
                        log::debug!(
                            "original body was a stream, total bytes: {:?}",
                            this.body_accum.size()
                        );
                        *this.eof = true;
                        let uri = req.uri().to_string();
                        let transforms = this
                            .req
                            .app_data::<web::Data<RespModData>>()
                            .map(|t| t.get_ref());
                        process(this.body_accum.to_bytes(), uri, transforms, &this.indexes)
                    } else {
                        Poll::Ready(None)
                    }
                }
                Poll::Pending => {
                    log::trace!("Poll::Pending {:?}", req.uri());
                    Poll::Pending
                }
            };
        }
    }
}

fn process(
    bytes: Bytes,
    uri: String,
    transforms: Option<&RespModData>,
    indexes: &Vec<usize>,
) -> Poll<Option<Result<Bytes, Error>>> {
    let to_process = std::str::from_utf8(&bytes);
    if let Ok(str) = to_process {
        let string = String::from(str);
        if !indexes.is_empty() {
            log::debug!("processing indexes {:?} for `{}`", indexes, uri);
            let next = transforms
                .map(|trans| trans.process_str(string.clone(), indexes))
                .unwrap_or(String::new());
            return Poll::Ready(Some(Ok(Bytes::from(next))));
        }
        Poll::Ready(Some(Ok(Bytes::from(string))))
    } else {
        Poll::Ready(Some(Ok(bytes)))
    }
}

fn is_ws_req(req: &RequestHead) -> bool {
    req.uri
        .clone()
        .into_parts()
        .path_and_query
        .map(|pq| pq.as_str().starts_with("/__bs3/ws"))
        .unwrap_or(false)
}
