#![warn(missing_docs, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]
//#![deny(warnings)]
//! Enriches the `lambda` crate with [`http`](https://github.com/hyperium/http)
//! types targeting AWS
//! * [ALB](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/introduction.html)
//! * [API Gateway](https://docs.aws.amazon.com/apigateway/latest/developerguide/welcome.html) REST and HTTP API lambda integrations
//! * [VPC Lattice](https://docs.aws.amazon.com/vpc-lattice/latest/ug/lambda-functions.html)
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
pub use lambda_runtime::{self, service_fn, tower, BoxFuture, Context, Error, LambdaEvent, Service, SnapStartResource};
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
    response::{BodyConversionError, LambdaResponse},
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
pub use streaming::{run_with_streaming_response, streaming_runtime, StreamAdapter};
#[cfg(feature = "concurrency-tokio")]
pub use streaming::{run_with_streaming_response_concurrent, streaming_runtime_concurrent};

/// Type alias for `http::Request`s with a fixed [`Body`](enum.Body.html) type
pub type Request = http::Request<Body>;

/// Future used by [`Adapter`] to convert an [`IntoResponse`] into a [`LambdaResponse`].
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

// The public Adapter must preserve its handler's error type. The runtime helpers
// can use Diagnostic as an internal common error channel for conversion failures.
enum RuntimeTransformResponse<'a, R, E> {
    Request(RequestOrigin, RequestFuture<'a, R, E>),
    Response(RequestOrigin, ResponseFuture),
}

impl<R, E> Future for RuntimeTransformResponse<'_, R, E>
where
    R: IntoResponse,
    E: Into<Diagnostic>,
{
    type Output = Result<LambdaResponse, Diagnostic>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        match *self {
            RuntimeTransformResponse::Request(ref mut origin, ref mut request) => match request.as_mut().poll(cx) {
                Poll::Ready(Ok(resp)) => {
                    *self = RuntimeTransformResponse::Response(origin.clone(), resp.into_response());
                    self.poll(cx)
                }
                Poll::Ready(Err(err)) => Poll::Ready(Err(err.into())),
                Poll::Pending => Poll::Pending,
            },
            RuntimeTransformResponse::Response(ref mut origin, ref mut response) => match response.as_mut().poll(cx) {
                Poll::Ready(mut resp) => {
                    if let Some(error) = resp.extensions_mut().remove::<BodyConversionError>() {
                        return Poll::Ready(Err(Diagnostic {
                            error_type: error.error_type.to_owned(),
                            error_message: error.error_message,
                        }));
                    }

                    Poll::Ready(Ok(LambdaResponse::from_response(origin, resp)))
                }
                Poll::Pending => Poll::Pending,
            },
        }
    }
}

struct RuntimeAdapter<'a, R, S> {
    service: S,
    _phantom_data: PhantomData<&'a R>,
}

impl<'a, R, S> Clone for RuntimeAdapter<'a, R, S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, R, S, E> From<S> for RuntimeAdapter<'a, R, S>
where
    S: Service<Request, Response = R, Error = E>,
    S::Future: Send + 'a,
    R: IntoResponse,
    E: Into<Diagnostic>,
{
    fn from(service: S) -> Self {
        Self {
            service,
            _phantom_data: PhantomData,
        }
    }
}

impl<'a, R, S, E> Service<LambdaEvent<LambdaRequest>> for RuntimeAdapter<'a, R, S>
where
    S: Service<Request, Response = R, Error = E>,
    S::Future: Send + 'a,
    R: IntoResponse,
    E: Into<Diagnostic>,
{
    type Response = LambdaResponse;
    type Error = Diagnostic;
    type Future = RuntimeTransformResponse<'a, R, E>;

    fn poll_ready(&mut self, cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: LambdaEvent<LambdaRequest>) -> Self::Future {
        let LambdaEvent { payload, context } = req;
        let request_origin = payload.request_origin();
        let mut event: Request = payload.into();
        update_xray_trace_id_header(event.headers_mut(), &context);
        let fut = Box::pin(self.service.call(event.with_lambda_context(context)));

        RuntimeTransformResponse::Request(request_origin, fut)
    }
}

/// Wraps a `Service<Request>` in a `Service<LambdaEvent<Request>>`
///
/// This adapter preserves the wrapped service's error type. Response body conversion
/// failures are returned as deterministic HTTP 500 responses.
#[non_exhaustive]
#[doc(hidden)]
pub struct Adapter<'a, R, S> {
    service: S,
    _phantom_data: PhantomData<&'a R>,
}

impl<'a, R, S> Clone for Adapter<'a, R, S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            _phantom_data: PhantomData,
        }
    }
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
        let LambdaEvent { payload, context } = req;
        let request_origin = payload.request_origin();
        let mut event: Request = payload.into();
        update_xray_trace_id_header(event.headers_mut(), &context);
        let fut = Box::pin(self.service.call(event.with_lambda_context(context)));

        TransformResponse::Request(request_origin, fut)
    }
}

/// Starts the Lambda Rust runtime and begins polling for events on the [Lambda
/// Runtime APIs](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
///
/// This takes care of transforming the LambdaEvent into a [`Request`] and then
/// converting the result into a `LambdaResponse`.
///
/// # Managed concurrency
/// If `AWS_LAMBDA_MAX_CONCURRENCY` is set, a warning is logged.
/// If your handler can satisfy `Clone + Send + 'static`,
/// prefer [`run_concurrent`] (requires the `concurrency-tokio` feature),
/// which honors managed concurrency and falls back to sequential behavior when
/// unset.
///
/// # Panics
///
///  This function panics if required Lambda environment variables are missing
/// (`AWS_LAMBDA_FUNCTION_NAME`, `AWS_LAMBDA_FUNCTION_MEMORY_SIZE`,
/// `AWS_LAMBDA_FUNCTION_VERSION`, `AWS_LAMBDA_RUNTIME_API`).
pub async fn run<'a, R, S, E>(handler: S) -> Result<(), Error>
where
    S: Service<Request, Response = R, Error = E>,
    S::Future: Send + 'a,
    R: IntoResponse,
    E: std::fmt::Debug + Into<Diagnostic>,
{
    lambda_runtime::run(RuntimeAdapter::from(handler)).await
}

/// Starts the Lambda Rust runtime and begins polling for events on the [Lambda
/// Runtime APIs](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
///
/// This takes care of transforming the LambdaEvent into a [`Request`] and then
/// converting the result into a `LambdaResponse`.
///
/// # Managed concurrency
///
/// When `AWS_LAMBDA_MAX_CONCURRENCY` is set to a value greater than 1, this
/// function spawns multiple tokio worker tasks to handle concurrent invocations.
/// When the environment variable is unset or `<= 1`, it falls back to
/// sequential behavior, so the same handler can run on both classic Lambda
/// and Lambda Managed Instances.
///
/// # Panics
///
/// This function panics if:
/// - Called outside of a Tokio runtime with `AWS_LAMBDA_MAX_CONCURRENCY > 1`
/// - Required Lambda environment variables are missing (`AWS_LAMBDA_FUNCTION_NAME`,
///   `AWS_LAMBDA_FUNCTION_MEMORY_SIZE`, `AWS_LAMBDA_FUNCTION_VERSION`,
///   `AWS_LAMBDA_RUNTIME_API`)
#[cfg(feature = "concurrency-tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "concurrency-tokio")))]
pub async fn run_concurrent<R, S, E>(handler: S) -> Result<(), Error>
where
    S: Service<Request, Response = R, Error = E> + Clone + Send + 'static,
    S::Future: Send + 'static,
    R: IntoResponse + Send + Sync + 'static,
    E: std::fmt::Debug + Into<Diagnostic> + Send + 'static,
{
    lambda_runtime::run_concurrent(RuntimeAdapter::from(handler)).await
}

/// Returns a configured [`Runtime`](lambda_runtime::Runtime) wrapping the given
/// handler, without starting the event loop.
///
/// Use this when you need to register
/// [`SnapStartResource`]s for the SnapStart
/// snapshot/restore lifecycle. For the common case where no custom hooks are
/// needed, use [`run()`] instead — SnapStart support works automatically.
///
/// # Example
///
/// ```no_run
/// use lambda_http::{service_fn, BoxFuture, Error, Request, SnapStartResource};
/// use std::sync::Arc;
///
/// struct Pool;
/// impl SnapStartResource for Pool {
///     fn before_snapshot(&self) -> BoxFuture<'_, Result<(), Error>> {
///         Box::pin(async move { Ok(()) })
///     }
///     fn after_restore(&self) -> BoxFuture<'_, Result<(), Error>> {
///         Box::pin(async move { Ok(()) })
///     }
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let pool = Arc::new(Pool);
///     let runtime = lambda_http::runtime(service_fn(handler))
///         .register_snapstart_resource(pool.clone());
///
///     runtime.run().await?;
///     Ok(())
/// }
///
/// async fn handler(_req: Request) -> Result<&'static str, std::convert::Infallible> {
///     Ok("hello")
/// }
/// ```
///
/// # Panics
///
/// This function panics if required Lambda environment variables are missing
/// (`AWS_LAMBDA_FUNCTION_NAME`, `AWS_LAMBDA_FUNCTION_MEMORY_SIZE`,
/// `AWS_LAMBDA_FUNCTION_VERSION`, `AWS_LAMBDA_RUNTIME_API`).
pub fn runtime<R, S, E>(
    handler: S,
) -> lambda_runtime::Runtime<
    impl lambda_runtime::Service<lambda_runtime::LambdaInvocation, Response = (), Error = lambda_runtime::Error>,
>
where
    S: Service<Request, Response = R, Error = E> + Send + 'static,
    S::Future: Send + 'static,
    R: IntoResponse + Send + Sync + 'static,
    E: std::fmt::Debug + Into<Diagnostic> + Send + 'static,
{
    lambda_runtime::Runtime::new(RuntimeAdapter::from(handler))
}

/// Returns a configured [`Runtime`](lambda_runtime::Runtime) wrapping the given
/// handler for concurrent execution, without starting the event loop.
///
/// This is the concurrent variant of [`runtime()`]. Use it when you need SnapStart
/// hooks AND your handler supports concurrent invocations.
///
/// # Panics
///
/// This function panics if required Lambda environment variables are missing.
#[cfg(feature = "concurrency-tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "concurrency-tokio")))]
pub fn runtime_concurrent<R, S, E>(
    handler: S,
) -> lambda_runtime::Runtime<
    impl lambda_runtime::Service<
            lambda_runtime::LambdaInvocation,
            Response = (),
            Error = lambda_runtime::Error,
            Future: Send,
        > + Clone
        + Send
        + 'static,
>
where
    S: Service<Request, Response = R, Error = E> + Clone + Send + 'static,
    S::Future: Send + 'static,
    R: IntoResponse + Send + Sync + 'static,
    E: std::fmt::Debug + Into<Diagnostic> + Send + 'static,
{
    lambda_runtime::Runtime::new(RuntimeAdapter::from(handler))
}

// In concurrent mode we must use the per-request context.
fn update_xray_trace_id_header(headers: &mut http::HeaderMap, context: &Context) {
    if let Some(trace_id) = context.xray_trace_id.as_deref() {
        if let Ok(header_value) = http::HeaderValue::from_str(trace_id) {
            headers.insert(http::header::HeaderName::from_static("x-amzn-trace-id"), header_value);
        }
    }
}

#[cfg(test)]
mod test_adapter {
    use bytes::Bytes;
    use futures_util::stream;
    use http_body::Frame;
    use http_body_util::StreamBody;
    use std::{
        io::{self, ErrorKind},
        task::{Context, Poll},
    };

    use crate::{
        aws_lambda_events::apigw::ApiGatewayV2httpRequest,
        http::{Response, StatusCode},
        lambda_runtime::LambdaEvent,
        request::LambdaRequest,
        response::LambdaResponse,
        tower::{util::BoxService, Service, ServiceBuilder, ServiceExt},
        Adapter, Body, Request, RuntimeAdapter,
    };

    fn fallible_body() -> impl http_body::Body<Data = Bytes, Error = io::Error> + Unpin {
        StreamBody::new(stream::iter([
            Ok(Frame::data(Bytes::from_static(b"partial response"))),
            Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "simulated truncated response body",
            )),
        ]))
    }

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

    #[tokio::test]
    async fn runtime_adapter_propagates_body_errors() {
        for content_type in ["text/plain; charset=utf-8", "application/octet-stream"] {
            let handler = crate::service_fn(move |_event: Request| async move {
                Ok::<_, std::convert::Infallible>(
                    Response::builder()
                        .header(http::header::CONTENT_TYPE, content_type)
                        .body(fallible_body())
                        .expect("unable to build http::Response"),
                )
            });
            let event = LambdaEvent::new(
                LambdaRequest::ApiGatewayV2(ApiGatewayV2httpRequest::default()),
                crate::Context::default(),
            );

            let error = RuntimeAdapter::from(handler)
                .oneshot(event)
                .await
                .expect_err("body collection error should be propagated");

            assert_eq!(error.error_type, std::any::type_name::<io::Error>());
            assert!(error.error_message.contains("simulated truncated response body"));
        }
    }

    async fn http_handler(_req: Request) -> Result<&'static str, std::convert::Infallible> {
        Ok("hello")
    }

    /// `runtime()` accepts a (non-`Clone`) handler for the sequential path. This
    /// is a compile-time check that the helper's bounds are satisfiable.
    #[test]
    fn runtime_helper_accepts_handler() {
        let _ = || {
            let _runtime = crate::runtime(crate::service_fn(http_handler));
        };
    }

    /// `runtime_concurrent()` requires a `Clone` handler (Lambda Managed
    /// Instances). `service_fn` over an `async fn` is `Clone`, so this compiles;
    /// it guards against the `Clone` bound regressing on the concurrent helper.
    ///
    /// Crucially, this also calls `.run_concurrent()` on the result: that method
    /// requires the wrapped service to be `Clone + Send + 'static` with a `Send`
    /// future, so this compiling proves the helper's return type exposes those
    /// bounds (without them, the chained call would not type-check).
    #[cfg(feature = "concurrency-tokio")]
    #[test]
    fn runtime_concurrent_helper_exposes_lmi_bounds() {
        let _ = || async {
            let _ = crate::runtime_concurrent(crate::service_fn(http_handler))
                .run_concurrent()
                .await;
        };
    }
}
