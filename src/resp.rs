use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::body::{BodySize, MessageBody, ResponseBody};
use actix_web::web::{Bytes, BytesMut};
use bytes::Buf;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, Ready};
use futures::{TryStreamExt, StreamExt};
use actix_web::guard::Guard;
use actix_web::dev::{RequestHead, ResponseHead};

trait RespGuard {
    fn check(&self, req_head: &RequestHead, _res_head: &mut ResponseHead) -> bool {
        if req_head.headers.contains_key("referer") {
            return false
        }
        return true
    }
}

struct ScriptTag;
impl RespGuard for ScriptTag {
    fn check(&self, req_head: &RequestHead, res_head: &mut ResponseHead) -> bool {
        if req_head.headers.contains_key("accept") {
            if req_head.headers.get("accept").expect("guarded").to_str().expect("ed").contains("text/html") {
                return true
            } else {
                println!("not doing {:#?}", res_head)
            }
        }
        return false
    }
}

pub struct Logging;

impl<S: 'static, B> Transform<S> for Logging
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

impl<S, B> Service for LoggingMiddleware<S>
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
            let head2 = req.head();
            res.map_body(move |head, body| {
                // println!("{:#?}", head2);
                // println!("{:#?}", head);
                let process = (ScriptTag).check(&head2, head);

                ResponseBody::Body(BodyLogger {
                    body,
                    body_accum: BytesMut::new(),
                    process,
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
    process: bool
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

        match this.body.poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                // pass-thru if not enabled for current request
                if !*this.process {
                    return Poll::Ready(Some(Ok(chunk)))
                }
                this.body_accum.extend_from_slice(&chunk);
                println!("this.body_accum = {:?}\n\
                          this.body       = {:?}", this.body_accum.size(), size1);
                if this.body_accum.size() == size1 {
                    let bytes = this.body_accum.to_bytes();
                    Poll::Ready(Some(Ok(bytes)))
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                if *this.process {
                    println!("🥰 done");
                }
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
