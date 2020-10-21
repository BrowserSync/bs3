use log::debug;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::body::{BodySize, MessageBody, ResponseBody};
use actix_web::web::{Bytes, BytesMut};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, web, Error, HttpRequest};
use bytes::Buf;
use futures::future::{ok, Ready};

use actix_web::dev::RequestHead;

pub trait RespMod {
    fn process_str(&self, resp: String) -> String {
        resp
    }
    fn guard(&self, req_head: &RequestHead) -> bool {
        if req_head.headers.contains_key("referer") {
            return false;
        }
        return true;
    }
}

pub trait RespModDataTrait {
    fn indexes(&self, req_head: &RequestHead) -> Vec<usize>;
    fn process_str(&self, input: String, indexes: &Vec<usize>) -> String;
}

pub struct RespModData {
    pub items: Vec<Box<dyn RespMod>>,
}

impl RespModDataTrait for RespModData {
    fn indexes(&self, req_head: &RequestHead) -> Vec<usize> {
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                if item.guard(&req_head) {
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
                let head = req.head();
                let transforms = req
                    .app_data::<web::Data<RespModData>>()
                    .map(|t| t.get_ref());
                let indexes: Vec<usize> = transforms
                    .map(|trans| trans.indexes(&head))
                    .unwrap_or(vec![]);
                ResponseBody::Body(BodyLogger {
                    body,
                    body_accum: BytesMut::new(),
                    process: !indexes.is_empty(),
                    indexes,
                    req,
                })
            })
        }))
    }
}

#[pin_project::pin_project(PinnedDrop)]
pub struct BodyLogger<B> {
    #[pin]
    body: ResponseBody<B>,
    body_accum: BytesMut,
    process: bool,
    indexes: Vec<usize>,
    req: HttpRequest,
}

#[pin_project::pinned_drop]
impl<B> PinnedDrop for BodyLogger<B> {
    fn drop(self: Pin<&mut Self>) {
        // println!("response body: {:?}", self.body_accum);
    }
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
        let this = self.project();
        let size1 = this.body.size().clone();
        let transforms = this
            .req
            .app_data::<web::Data<RespModData>>()
            .map(|t| t.get_ref());

        match this.body.poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                if !*this.process {
                    return Poll::Ready(Some(Ok(chunk)));
                }
                this.body_accum.extend_from_slice(&chunk);
                debug!(
                    "this.body_accum = {:?}, this.body = {:?}",
                    this.body_accum.size(),
                    size1
                );
                if this.body_accum.size() == size1 {
                    process(this.body_accum.to_bytes(), transforms, &this.indexes)
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

fn process(
    bytes: Bytes,
    transforms: Option<&RespModData>,
    indexes: &Vec<usize>,
) -> Poll<Option<Result<Bytes, Error>>> {
    let to_process = std::str::from_utf8(&bytes);
    if let Ok(str) = to_process {
        let string = String::from(str);
        if !indexes.is_empty() {
            log::debug!("processing indexes {:?}", indexes);
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
