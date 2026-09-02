use crate::{
    constants::LAMBDA_RUNTIME_INVOCATION_ID,
    deserializer,
    rate_limiter::rate_limited,
    requests::{EventCompletionRequest, IntoRequest},
    runtime::LambdaInvocation,
    Diagnostic, EventErrorRequest, IntoFunctionResponse, LambdaEvent,
};
use futures::{ready, Stream};
use lambda_runtime_api_client::{body::Body, BoxError};
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, future::Future, marker::PhantomData, pin::Pin, task, time::Duration};
use tower::Service;
use tracing::{error, trace, warn};

const MALFORMED_INVOCATION_ID_LOG_INTERVAL: Duration = Duration::from_secs(60);

/// Tower service that turns the result or an error of a handler function into a Lambda Runtime API
/// response.
///
/// This type is only meant for internal use in the Lambda runtime crate. The service augments both
/// inputs and outputs: the input is converted from a [LambdaInvocation] into a [LambdaEvent]
/// while any errors encountered during the conversion are turned into error responses. The service
/// outputs either a HTTP request to send to the Lambda Runtime API or a boxed error which ought to
/// be propagated to the caller to terminate the runtime.
pub struct RuntimeApiResponseService<
    S,
    EventPayload,
    Response,
    BufferedResponse,
    StreamingResponse,
    StreamItem,
    StreamError,
> {
    inner: S,
    _phantom: PhantomData<(
        EventPayload,
        Response,
        BufferedResponse,
        StreamingResponse,
        StreamItem,
        StreamError,
    )>,
}

impl<S, EventPayload, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError>
    RuntimeApiResponseService<S, EventPayload, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError>
{
    pub fn new(inner: S) -> Self {
        Self {
            inner,
            _phantom: PhantomData,
        }
    }
}

impl<S, EventPayload, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError> Clone
    for RuntimeApiResponseService<
        S,
        EventPayload,
        Response,
        BufferedResponse,
        StreamingResponse,
        StreamItem,
        StreamError,
    >
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<S, EventPayload, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError> Service<LambdaInvocation>
    for RuntimeApiResponseService<
        S,
        EventPayload,
        Response,
        BufferedResponse,
        StreamingResponse,
        StreamItem,
        StreamError,
    >
where
    S: Service<LambdaEvent<EventPayload>, Response = Response, Error = Diagnostic>,
    EventPayload: for<'de> Deserialize<'de>,
    Response: IntoFunctionResponse<BufferedResponse, StreamingResponse>,
    BufferedResponse: Serialize,
    StreamingResponse: Stream<Item = Result<StreamItem, StreamError>> + Unpin + Send + 'static,
    StreamItem: Into<bytes::Bytes> + Send,
    StreamError: Into<BoxError> + Send + Debug,
{
    type Response = http::Request<Body>;
    type Error = BoxError;
    type Future =
        RuntimeApiResponseFuture<S::Future, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|err| BoxError::from(format!("{}: {}", err.error_type, err.error_message)))
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        #[cfg(debug_assertions)]
        if req.parts.status.is_server_error() {
            error!("Lambda Runtime server returned an unexpected error");
            return RuntimeApiResponseFuture::Ready(Box::new(Some(Err(req.parts.status.to_string().into()))));
        }

        // Utility closure to propagate potential error from conditionally executed trace
        let trace_fn = || {
            trace!(
                body = std::str::from_utf8(&req.body)?,
                "raw JSON event received from Lambda"
            );
            Ok(())
        };
        if let Err(err) = trace_fn() {
            error!(error = ?err, "Failed to parse raw JSON event received from Lambda. The handler will not be called. Log at TRACE level to see the payload.");
            return RuntimeApiResponseFuture::Ready(Box::new(Some(Err(err))));
        };

        let request_id = req.context.request_id.clone();

        // The invocation ID assigned by the Lambda runtime for cross-wiring protection.
        // Echoed back on `/response` and `/error` to allow RAPID to reject stale responses
        // from timed-out invocations. `None` when running against older RAPID versions
        // that don't send this header.
        let invocation_id = match req.parts.headers.get(LAMBDA_RUNTIME_INVOCATION_ID) {
            Some(value) => match value.to_str() {
                Ok(value) => Some(value.to_owned()),
                Err(error) => {
                    rate_limited!(MALFORMED_INVOCATION_ID_LOG_INTERVAL, {
                        warn!(
                            error = ?error,
                            rate_limit_interval_ms = MALFORMED_INVOCATION_ID_LOG_INTERVAL.as_millis(),
                            "Ignoring malformed Lambda runtime invocation ID header"
                        );
                    });
                    None
                }
            },
            None => None,
        };

        let lambda_event = match deserializer::deserialize::<EventPayload>(&req.body, req.context) {
            Ok(lambda_event) => lambda_event,
            Err(err) => match build_event_error_request(request_id, invocation_id, err) {
                Ok(request) => return RuntimeApiResponseFuture::Ready(Box::new(Some(Ok(request)))),
                Err(err) => {
                    error!(error = ?err, "failed to build error response for Lambda Runtime API");
                    return RuntimeApiResponseFuture::Ready(Box::new(Some(Err(err))));
                }
            },
        };

        // Once the handler input has been generated successfully, pass it through to inner services
        // allowing processing both before reaching the handler function and after the handler completes.
        let fut = self.inner.call(lambda_event);
        RuntimeApiResponseFuture::Future(fut, request_id, invocation_id, PhantomData)
    }
}

fn build_event_error_request<T>(
    request_id: String,
    invocation_id: Option<String>,
    err: T,
) -> Result<http::Request<Body>, BoxError>
where
    T: Into<Diagnostic> + Debug,
{
    error!(error = ?err, "Request payload deserialization into LambdaEvent<T> failed. The handler will not be called. Log at TRACE level to see the payload.");
    EventErrorRequest::new(&request_id, invocation_id.as_deref(), err).into_req()
}

#[pin_project(project = RuntimeApiResponseFutureProj)]
pub enum RuntimeApiResponseFuture<F, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError> {
    Future(
        #[pin] F,
        String,
        Option<String>,
        PhantomData<(
            (),
            Response,
            BufferedResponse,
            StreamingResponse,
            StreamItem,
            StreamError,
        )>,
    ),
    /// This variant is used in case the invocation fails to be processed into an event.
    /// We box it to avoid bloating the size of the more likely variant, which is
    /// the future that drives event processing.
    Ready(Box<Option<Result<http::Request<Body>, BoxError>>>),
}

impl<F, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError> Future
    for RuntimeApiResponseFuture<F, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError>
where
    F: Future<Output = Result<Response, Diagnostic>>,
    Response: IntoFunctionResponse<BufferedResponse, StreamingResponse>,
    BufferedResponse: Serialize,
    StreamingResponse: Stream<Item = Result<StreamItem, StreamError>> + Unpin + Send + 'static,
    StreamItem: Into<bytes::Bytes> + Send,
    StreamError: Into<BoxError> + Send + Debug,
{
    type Output = Result<http::Request<Body>, BoxError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        task::Poll::Ready(match self.as_mut().project() {
            RuntimeApiResponseFutureProj::Future(fut, request_id, invocation_id, _) => match ready!(fut.poll(cx)) {
                Ok(ok) => EventCompletionRequest::new(request_id, invocation_id.as_deref(), ok).into_req(),
                Err(err) => EventErrorRequest::new(request_id, invocation_id.as_deref(), err).into_req(),
            },
            RuntimeApiResponseFutureProj::Ready(ready) => ready.take().expect("future polled after completion"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{constants::LAMBDA_RUNTIME_INVOCATION_ID, runtime::LambdaInvocation, Context};
    use http::{HeaderValue, Response};
    use serde_json::json;
    use tower::{service_fn, Service};

    #[tokio::test]
    async fn forwards_invocation_id_from_next_response_headers() {
        let mut response = Response::new(());
        response
            .headers_mut()
            .insert(LAMBDA_RUNTIME_INVOCATION_ID, HeaderValue::from_static("invocation-123"));
        let (parts, _) = response.into_parts();

        let mut service = RuntimeApiResponseService::new(service_fn(|_event: LambdaEvent<serde_json::Value>| async {
            Ok::<_, Diagnostic>(json!({"ok": true}))
        }));

        let request = service
            .call(LambdaInvocation {
                parts,
                body: bytes::Bytes::from_static(b"{}"),
                context: Context::default(),
            })
            .await
            .expect("response request should be created");

        assert_eq!(
            request.headers().get(LAMBDA_RUNTIME_INVOCATION_ID),
            Some(&HeaderValue::from_static("invocation-123")),
        );
    }

    #[tokio::test]
    async fn malformed_invocation_id_does_not_block_deserialization_error_response() {
        let mut response = Response::new(());
        response.headers_mut().insert(
            LAMBDA_RUNTIME_INVOCATION_ID,
            HeaderValue::from_bytes(&[0xff]).expect("header value should accept opaque bytes"),
        );
        let (parts, _) = response.into_parts();

        let mut service = RuntimeApiResponseService::new(service_fn(|_event: LambdaEvent<serde_json::Value>| async {
            Ok::<_, Diagnostic>(json!({"ok": true}))
        }));

        let request = service
            .call(LambdaInvocation {
                parts,
                body: bytes::Bytes::from_static(b"{"),
                context: Context {
                    request_id: "request-123".to_owned(),
                    ..Context::default()
                },
            })
            .await
            .expect("deserialization errors should produce an error request");

        assert_eq!(request.uri().path(), "/2018-06-01/runtime/invocation/request-123/error",);
        assert!(request.headers().get(LAMBDA_RUNTIME_INVOCATION_ID).is_none());
    }
}
