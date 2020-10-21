use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::body::{BodySize, MessageBody, ResponseBody};
use actix_web::web::{Bytes, BytesMut};
use bytes::Buf;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, web, HttpRequest};
use futures::future::{ok, Ready};
use futures::{TryStreamExt, StreamExt};
use actix_web::guard::Guard;
use actix_web::dev::{RequestHead, ResponseHead};
use crate::client::script::Script;

pub trait RespTransform: RespMod + RespGuard {}

fn indexes(items: &Vec<Box<dyn RespTransform>>, req_head: &RequestHead) -> Vec<usize> {
    items.iter().enumerate().filter_map(|(index, item)| {
        if item.check(&req_head) { Some(index) } else { None }
    }).collect()
}

fn try_indexes<'a>(items: Option<&'a Vec<Box<dyn RespTransform>>>, req_head: &RequestHead) -> Vec<usize> {
    if let Some(items) = items {
        return indexes(items, req_head);
    }
    return vec![]
}

pub trait RespMod {
    fn process_str(&self, resp: String) -> String {
        resp
    }
}

pub trait RespGuard {
    fn check(&self, req_head: &RequestHead) -> bool {
        if req_head.headers.contains_key("referer") {
            return false;
        }
        return true;
    }
}

// pub struct RespModData {
//     pub guard: Box<dyn RespGuard>,
//     pub process: Box<dyn RespMod>,
// }

// impl RespModMiddleware {
//     pub fn new(guard: Box<dyn RespGuard>, process: Box<dyn RespMod>) -> RespModData {
//         return RespModData { guard, process }
//     }
// }

pub struct RespModMiddleware;

impl<S: 'static, B> Transform<S> for RespModMiddleware
    where
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
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
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
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
    _t: PhantomData<(B, )>,
}

impl<S, B> Future for WrapperStream<S, B>
    where
        B: MessageBody,
        S: Service<Request=ServiceRequest, Response=ServiceResponse<B>, Error=Error>,
{
    type Output = Result<ServiceResponse<BodyLogger<B>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let res: Result<ServiceResponse<_>, _> = futures::ready!(self.project().fut.poll(cx));

        Poll::Ready(res.map(|res| {
            let req = res.request().clone();
            res.map_body(move |head, body| {
                // let head2 = req.head();
                // let tera = req.app_data::<web::Data<RespModData>>().map(|t| t.get_ref());
                // let process = if let Some(res) = tera {
                //     res.guard.check(head2)
                // } else { false };
                ResponseBody::Body(BodyLogger {
                    body,
                    body_accum: BytesMut::new(),
                    process: true,
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

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Error>>> {
        let this = self.project();
        let size1 = this.body.size().clone();
        let head = this.req.head();
        let transforms = this.req.app_data::<web::Data<Vec<Box<dyn RespTransform>>>>().map(|t| t.get_ref());
        let indexes = try_indexes(transforms, &head);

        match this.body.poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                if !*this.process {
                    return Poll::Ready(Some(Ok(chunk)));
                }
                this.body_accum.extend_from_slice(&chunk);
                println!("this.body_accum = {:?}\n\
                          this.body       = {:?}", this.body_accum.size(), size1);
                if this.body_accum.size() == size1 {
                    let bytes = this.body_accum.to_bytes();
                    let to_process = std::str::from_utf8(&bytes);
                    if let Ok(str) = to_process {
                        let mut string = String::from(str);
                        if !indexes.is_empty() {
                            println!("should process indexes {:#?}", indexes);
                            let next = indexes.iter()
                                .map(|index| {
                                    let item = transforms.expect("here").get(*index).expect("access by index");
                                    return item
                                })
                                .fold(string.clone(), |output, item| item.process_str(output));
                            // let processed = res.process.process_str(string);
                            return Poll::Ready(Some(Ok(Bytes::from(next))))
                        }
                        Poll::Ready(Some(Ok(Bytes::from(string))))
                    } else {
                        Poll::Ready(Some(Ok(bytes)))
                    }
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
