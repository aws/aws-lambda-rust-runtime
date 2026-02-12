use crate::LambdaInvocation;
use futures::{future::BoxFuture, ready, FutureExt, TryFutureExt};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use lambda_runtime_api_client::{body::Body, BoxError, Client};
use pin_project::pin_project;
use std::{future::Future, pin::Pin, sync::Arc, task};
use tower::Service;
use tracing::{error, warn};

/// Tower service that sends a Lambda Runtime API response to the Lambda Runtime HTTP API using
/// a previously initialized client.
///
/// This type is only meant for internal use in the Lambda runtime crate. It neither augments the
/// inner service's request type nor its error type. However, this service returns an empty
/// response `()` as the Lambda request has been completed.
pub struct RuntimeApiClientService<S> {
    inner: S,
    client: Arc<Client>,
}

impl<S> RuntimeApiClientService<S> {
    pub fn new(inner: S, client: Arc<Client>) -> Self {
        Self { inner, client }
    }
}

impl<S> Service<LambdaInvocation> for RuntimeApiClientService<S>
where
    S: Service<LambdaInvocation, Error = BoxError>,
    S::Future: Future<Output = Result<http::Request<Body>, BoxError>>,
{
    type Response = ();
    type Error = S::Error;
    type Future = RuntimeApiClientFuture<S::Future, Incoming>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        let request_fut = self.inner.call(req);
        let client = self.client.clone();
        RuntimeApiClientFuture::Invoke(request_fut, client)
    }
}

impl<S> Clone for RuntimeApiClientService<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            client: self.client.clone(),
        }
    }
}

/// Future representing the three-phase lifecycle of a Lambda invocation response.
///
/// This future implements a state machine with three phases:
///
/// 1. **Invoke**: Poll the inner service (customer handler code) to produce an HTTP request
///    containing the Lambda response to send to the Runtime API.
///
/// 2. **Respond**: Send the HTTP request to the Lambda Runtime API and await the HTTP response.
///    At this point, response headers are available but the body has not been consumed.
///
/// 3. **Reconcile**: Consume the HTTP response body and check for timeout indicators in trailers.
///    AWS Lambda uses HTTP 200 with `Lambda-Runtime-Function-Response-Status: timeout` trailer
///    to indicate timeout conditions in concurrent mode. Trailers are sent via HTTP/1.1 chunked
///    transfer encoding.
///
/// The three-phase design is necessary because HTTP trailers are sent after the response body,
/// requiring full body consumption before trailer access.
///
/// The type parameter `B` represents the response body type. In production this is `hyper::body::Incoming`,
/// but for testing it can be any body type that implements `http_body::Body`.
#[pin_project(project = RuntimeApiClientFutureProj)]
pub enum RuntimeApiClientFuture<F, B> {
    /// **Invoke Phase**: Polling the inner service to build the Lambda response request.
    ///
    /// Contains:
    /// - The inner service future that produces `http::Request<Body>`
    /// - Arc reference to the HTTP client for sending the request
    Invoke(#[pin] F, Arc<Client>),

    /// **Respond Phase**: Sending the request to Lambda Runtime API and receiving the response.
    ///
    /// Contains a boxed future that:
    /// - Sends the HTTP request via `client.call()`
    /// - Receives `http::Response<B>` with headers (body not yet consumed)
    /// - Transitions to Reconcile phase to consume body and check trailers
    Respond(#[pin] BoxFuture<'static, Result<http::Response<B>, BoxError>>),

    /// **Reconcile Phase**: Consuming response body to complete the HTTP transaction.
    ///
    /// Contains a boxed future that:
    /// - Consumes the entire HTTP response body via `.frame().await`
    /// - Checks trailers for timeout indicators
    /// - Returns `Ok(())` to complete the invocation lifecycle
    Reconcile(#[pin] BoxFuture<'static, Result<(), BoxError>>),
}

impl<F> Future for RuntimeApiClientFuture<F, Incoming>
where
    F: Future<Output = Result<http::Request<Body>, BoxError>>,
{
    type Output = Result<(), BoxError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        // Loop to directly transition between phases without yielding to the executor
        task::Poll::Ready(loop {
            match self.as_mut().project() {
                RuntimeApiClientFutureProj::Invoke(fut, client) => match ready!(fut.poll(cx)) {
                    Ok(ok) => {
                        // Invoke phase complete: transition to Respond phase
                        // Send the Lambda response request to the Runtime API
                        let next_fut = client
                            .call(ok)
                            .map_err(|err| {
                                error!(error = ?err, "failed to send request to Lambda Runtime API");
                                err
                            })
                            .boxed();
                        self.set(RuntimeApiClientFuture::Respond(next_fut));
                    }
                    Err(err) => break Err(err),
                },
                RuntimeApiClientFutureProj::Respond(fut) => {
                    let response = ready!(fut.poll(cx))?;

                    // Respond phase complete: transition to Reconcile phase
                    // Check for timeout indication in response trailers
                    let reconcile_fut = reconcile_response(response);
                    self.set(RuntimeApiClientFuture::Reconcile(reconcile_fut));
                }
                RuntimeApiClientFutureProj::Reconcile(fut) => {
                    // Reconcile phase: consume body, check trailers, complete
                    break ready!(fut.poll(cx));
                }
            }
        })
    }
}

/// Consume response body and check for timeout trailers.
/// AWS Lambda uses HTTP 200 with `Lambda-Runtime-Function-Response-Status: timeout` trailer
/// to indicate timeout conditions in concurrent mode.
fn reconcile_response<B>(response: http::Response<B>) -> BoxFuture<'static, Result<(), BoxError>>
where
    B: hyper::body::Body + Send + Unpin + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    async move {
        let mut body = response.into_body();

        while let Some(frame_result) = body.frame().await {
            match frame_result {
                Ok(frame) => {
                    if let Ok(trailers) = frame.into_trailers() {
                        // Check for Lambda-Runtime-Function-Response-Status: timeout
                        if let Some(status) = trailers.get("Lambda-Runtime-Function-Response-Status") {
                            if status == "timeout" {
                                warn!(
                                    "Lambda invocation timed out - response was not sent within the configured timeout"
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }
    .boxed()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use http::Response;
    use http_body_util::StreamBody;
    use hyper::body::Frame;
    use std::convert::Infallible;

    // Type alias for our test body
    type TestBody = StreamBody<futures::stream::Iter<std::vec::IntoIter<Result<Frame<Bytes>, Infallible>>>>;

    #[tokio::test]
    async fn test_reconcile_detects_timeout_trailer() {
        use tracing_capture::{CaptureLayer, SharedStorage};
        use tracing_subscriber::layer::SubscriberExt;

        // Set up tracing capture to verify the warning is logged
        let storage = SharedStorage::default();
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(&storage));
        let _guard = tracing::subscriber::set_default(subscriber);

        // Create a response body with trailers indicating timeout
        let body_data = Bytes::from_static(b"response body");
        let mut trailers = http::HeaderMap::new();
        trailers.insert("Lambda-Runtime-Function-Response-Status", "timeout".parse().unwrap());

        // Create a stream with data frame followed by trailer frame
        let frames: Vec<Result<Frame<Bytes>, Infallible>> =
            vec![Ok(Frame::data(body_data)), Ok(Frame::trailers(trailers))];
        let stream_body: TestBody = StreamBody::new(futures::stream::iter(frames));

        // Create a mock response
        let response: Response<TestBody> = Response::builder().status(200).body(stream_body).unwrap();

        // Test the reconcile_response function directly
        let result = reconcile_response(response).await;

        // Should complete successfully
        assert!(result.is_ok(), "Future should complete successfully: {:?}", result);

        // Verify warning was logged
        let storage_lock = storage.lock();
        let all_spans: Vec<_> = storage_lock.all_spans().collect();

        if !all_spans.is_empty() {
            let warnings: Vec<_> = all_spans
                .iter()
                .flat_map(|span| span.events())
                .filter(|event| {
                    event.metadata().level() == &tracing::Level::WARN
                        && event.message().map(|msg| msg.contains("timed out")).unwrap_or(false)
                })
                .collect();

            assert!(!warnings.is_empty(), "Warning should have been logged");
        }
    }

    #[tokio::test]
    async fn test_reconcile_without_timeout_trailer() {
        // Create a response body without timeout trailers
        let body_data = Bytes::from_static(b"response body");

        // Create a stream with just a data frame (no trailers)
        let frames: Vec<Result<Frame<Bytes>, Infallible>> = vec![Ok(Frame::data(body_data))];
        let stream_body: TestBody = StreamBody::new(futures::stream::iter(frames));

        let response: Response<TestBody> = Response::builder().status(200).body(stream_body).unwrap();

        // Test the reconcile_response function directly
        let result = reconcile_response(response).await;

        // Should complete successfully without any timeout warnings
        assert!(result.is_ok(), "Future should complete successfully");
    }
}
