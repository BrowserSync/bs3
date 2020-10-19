use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::body::{BodySize, MessageBody, ResponseBody};
use actix_web::web::{Bytes, BytesMut};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, Ready};
use bytes::Buf;

pub struct Logging;

impl<S: 'static, B> Transform<S> for Logging
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

impl<S, B> Service for LoggingMiddleware<S>
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
        let res = futures::ready!(self.project().fut.poll(cx));

        Poll::Ready(res.map(|res| {
            res.map_body(move |_, body| {
                ResponseBody::Body(BodyLogger {
                    body,
                    body_accum: BytesMut::new(),
                    sent: false,
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
    sent: bool,
}

#[pin_project::pinned_drop]
impl<B> PinnedDrop for BodyLogger<B> {
    fn drop(self: Pin<&mut Self>) {
        println!("response body: {:?}", self.body_accum);
    }
}

impl<B: MessageBody> MessageBody for BodyLogger<B> {
    fn size(&self) -> BodySize {
        BodySize::Stream
    }

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Error>>> {
        let this = self.project();
        if *this.sent {
            println!("poll sent");
            Poll::Ready(None)
        } else {
            println!("poll not sent");
            match this.body.poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    if *this.sent {
                        println!("+++completing+++");
                        Poll::Pending
                    } else {
                        println!("chunk, saving but sending Pending back");
                        this.body_accum.extend_from_slice(&chunk);
                        // Poll::Ready(Some(Ok(Bytes::new())))
                        Poll::Pending
                    }
                }
                Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => {
                    if *this.sent {
                        println!("+++completing+++");
                        Poll::Ready(None)
                    } else {
                        println!("sending the chunk = {:?}", this.body_accum.size());
                        *this.sent = true;
                        // let b = this.body_accum.replace("</body>", "");
                        // let st = *this.body_accum.to_str();
                        let st = std::str::from_utf8(this.body_accum).expect("workd");
                        let next = String::from(st).replace("</body>", "<script>alert('Yay!')</script></body>");
                        Poll::Ready(Some(Ok(Bytes::from(next))))
                    }
                },
                Poll::Pending => Poll::Pending,
            }
        }


        // let b = Bytes::new();
        // Poll::Ready(Some(Ok(b)))
    }
}
