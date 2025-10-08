#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//#![deny(warnings)]
//! Enriches the `lambda` crate with [`http`](https://github.com/hyperium/http)
//! types targeting AWS [ALB](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/introduction.html), [API Gateway](https://docs.aws.amazon.com/apigateway/latest/developerguide/welcome.html) REST and HTTP API lambda integrations.
//!
//! This crate abstracts over all of these trigger events using standard [`http`](https://github.com/hyperium/http) types minimizing the mental overhead
//! of understanding the nuances and variation between trigger details allowing you to focus more on your application while also giving you to the maximum flexibility to
//! transparently use whichever lambda trigger suits your application and cost optimizations best.
//!
//! # Examples
//!
//! ## Hello World
//!
//! The following example is how you would structure your Lambda such that you have a `main` function where you explicitly invoke
//! `lambda_http::run` in combination with the [`service_fn`](fn.service_fn.html) function. This pattern allows you to utilize global initialization
//! of tools such as loggers, to use on warm invokes to the same Lambda function after the first request, helping to reduce the latency of
//! your function's execution path.
//!
//! ```rust,no_run
//! use lambda_http::{service_fn, Error};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     // initialize dependencies once here for the lifetime of your
//!     // lambda task
//!     lambda_http::run(service_fn(|request| async {
//!         Result::<&str, std::convert::Infallible>::Ok("👋 world!")
//!     })).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Leveraging trigger provided data
//!
//! You can also access information provided directly from the underlying trigger events,
//! like query string parameters, or Lambda function context, with the [`RequestExt`] trait.
//!
//! ```rust,no_run
//! use lambda_http::{service_fn, Error, RequestExt, IntoResponse, Request};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!     lambda_http::run(service_fn(hello)).await?;
//!     Ok(())
//! }
//!
//! async fn hello(
//!     request: Request
//! ) -> Result<impl IntoResponse, std::convert::Infallible> {
//!     let _context = request.lambda_context_ref();
//!
//!     Ok(format!(
//!         "hello {}",
//!         request
//!             .query_string_parameters_ref()
//!             .and_then(|params| params.first("name"))
//!             .unwrap_or_else(|| "stranger")
//!     ))
//! }
//! ```

// only externed because maplit doesn't seem to play well with 2018 edition imports
#[cfg(test)]
#[macro_use]
extern crate maplit;

pub use http::{self, Response};
/// Utilities to initialize and use `tracing` and `tracing-subscriber` in Lambda Functions.
#[cfg(feature = "tracing")]
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
pub use lambda_runtime::tracing;
use lambda_runtime::Diagnostic;
pub use lambda_runtime::{self, service_fn, tower, Context, Error, LambdaEvent, Service};
use request::RequestFuture;
use response::ResponseFuture;

mod deserializer;
pub mod ext;
pub mod request;
mod response;
pub use crate::{
    ext::{RequestExt, RequestPayloadExt},
    response::IntoResponse,
};
use crate::{
    request::{LambdaRequest, RequestOrigin},
    response::LambdaResponse,
};

// Reexported in its entirety, regardless of what feature flags are enabled
// because working with many of these types requires other types in, or
// reexported by, this crate.
pub use aws_lambda_events;

pub use aws_lambda_events::encodings::Body;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context as TaskContext, Poll},
};

mod streaming;
pub use streaming::{run_with_streaming_response, StreamAdapter};

/// Type alias for `http::Request`s with a fixed [`Body`](enum.Body.html) type
pub type Request = http::Request<Body>;

/// Future that will convert an [`IntoResponse`] into an actual [`LambdaResponse`]
///
/// This is used by the `Adapter` wrapper and is completely internal to the `lambda_http::run` function.
#[non_exhaustive]
#[doc(hidden)]
pub enum TransformResponse<'a, R, E> {
    Request(RequestOrigin, RequestFuture<'a, R, E>),
    Response(RequestOrigin, ResponseFuture),
}

impl<R, E> Future for TransformResponse<'_, R, E>
where
    R: IntoResponse,
{
    type Output = Result<LambdaResponse, E>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        match *self {
            TransformResponse::Request(ref mut origin, ref mut request) => match request.as_mut().poll(cx) {
                Poll::Ready(Ok(resp)) => {
                    *self = TransformResponse::Response(origin.clone(), resp.into_response());
                    self.poll(cx)
                }
                Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                Poll::Pending => Poll::Pending,
            },
            TransformResponse::Response(ref mut origin, ref mut response) => match response.as_mut().poll(cx) {
                Poll::Ready(resp) => Poll::Ready(Ok(LambdaResponse::from_response(origin, resp))),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}

/// Wraps a `Service<Request>` in a `Service<LambdaEvent<Request>>`
///
/// This is completely internal to the `lambda_http::run` function.
#[non_exhaustive]
#[doc(hidden)]
pub struct Adapter<'a, R, S> {
    service: S,
    _phantom_data: PhantomData<&'a R>,
}

impl<'a, R, S, E> From<S> for Adapter<'a, R, S>
where
    S: Service<Request, Response = R, Error = E>,
    S::Future: Send + 'a,
    R: IntoResponse,
{
    fn from(service: S) -> Self {
        Adapter {
            service,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, R, S, E> Service<LambdaEvent<LambdaRequest>> for Adapter<'a, R, S>
where
    S: Service<Request, Response = R, Error = E>,
    S::Future: Send + 'a,
    R: IntoResponse,
{
    type Response = LambdaResponse;
    type Error = E;
    type Future = TransformResponse<'a, R, Self::Error>;

    fn poll_ready(&mut self, cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaEvent<LambdaRequest>) -> Self::Future {
        let request_origin = req.payload.request_origin();
        let event: Request = req.payload.into();
        let fut = Box::pin(self.service.call(event.with_lambda_context(req.context)));

        TransformResponse::Request(request_origin, fut)
    }
}

/// Starts the Lambda Rust runtime and begins polling for events on the [Lambda
/// Runtime APIs](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
///
/// This takes care of transforming the LambdaEvent into a [`Request`] and then
/// converting the result into a `LambdaResponse`.
pub async fn run<'a, R, S, E>(handler: S) -> Result<(), Error>
where
    S: Service<Request, Response = R, Error = E>,
    S::Future: Send + 'a,
    R: IntoResponse,
    E: std::fmt::Debug + Into<Diagnostic>,
{
    lambda_runtime::run(Adapter::from(handler)).await
}

#[cfg(test)]
mod test_adapter {
    use std::task::{Context, Poll};

    use crate::{
        http::{Response, StatusCode},
        lambda_runtime::LambdaEvent,
        request::LambdaRequest,
        response::LambdaResponse,
        tower::{util::BoxService, Service, ServiceBuilder, ServiceExt},
        Adapter, Body, Request,
    };

    // A middleware that logs requests before forwarding them to another service
    struct LogService<S> {
        inner: S,
    }

    impl<S> Service<LambdaEvent<LambdaRequest>> for LogService<S>
    where
        S: Service<LambdaEvent<LambdaRequest>>,
    {
        type Response = S::Response;
        type Error = S::Error;
        type Future = S::Future;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, event: LambdaEvent<LambdaRequest>) -> Self::Future {
            // Log the request
            println!("Lambda event: {event:#?}");

            self.inner.call(event)
        }
    }

    /// This tests that `Adapter` can be used in a `tower::Service` where the user
    /// may require additional middleware between `lambda_runtime::run` and where
    /// the `LambdaEvent` is converted into a `Request`.
    #[test]
    fn adapter_is_boxable() {
        let _service: BoxService<LambdaEvent<LambdaRequest>, LambdaResponse, http::Error> = ServiceBuilder::new()
            .layer_fn(|service| {
                // This could be any middleware that logs, inspects, or manipulates
                // the `LambdaEvent` before it's converted to a `Request` by `Adapter`.

                LogService { inner: service }
            })
            .layer_fn(Adapter::from)
            .service_fn(|_event: Request| async move { Response::builder().status(StatusCode::OK).body(Body::Empty) })
            .boxed();
    }
}
