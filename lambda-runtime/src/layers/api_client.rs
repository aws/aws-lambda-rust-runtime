use crate::LambdaInvocation;
use futures::{future::BoxFuture, ready, FutureExt, TryFutureExt};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use lambda_runtime_api_client::{body::Body, BoxError, Client};
use pin_project::pin_project;
use std::{future::Future, pin::Pin, sync::Arc, task};
use tokio::runtime::{Handle, Runtime};
use tower::Service;
use tracing::error;

/// Tower service that sends a Lambda Runtime API response to the Lambda Runtime HTTP API using
/// a previously initialized client.
///
/// This type is only meant for internal use in the Lambda runtime crate. It neither augments the
/// inner service's request type nor its error type. However, this service returns an empty
/// response `()` as the Lambda request has been completed.
pub struct RuntimeApiClientService<S> {
    inner: S,
    client: Arc<Client>,
    /// Handle to the tokio runtime used to spawn background tasks (e.g. logging error bodies).
    /// Obtained from the ambient runtime at construction time, or from a self-managed fallback.
    handle: Handle,
    /// Keeps the self-managed fallback runtime alive for as long as this service exists.
    /// `None` when constructed inside an existing tokio context (the common case).
    /// Must not be dropped before `handle`, as dropping the runtime invalidates the handle.
    fallback_runtime: Option<Arc<Runtime>>,
}

impl<S> RuntimeApiClientService<S> {
    pub fn new(inner: S, client: Arc<Client>) -> Self {
        let (handle, fallback) = match Handle::try_current() {
            Ok(h) => (h, None),
            Err(_) => {
                // Fallback: build a minimal current-thread runtime and keep it alive alongside
                // the service. This should only happen outside of a tokio context (e.g. some tests).
                let rt = Arc::new(
                    tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("failed to build fallback tokio runtime"),
                );
                let h = rt.handle().clone();
                (h, Some(rt))
            }
        };
        Self {
            inner,
            client,
            handle,
            fallback_runtime: fallback,
        }
    }
}

impl<S> Service<LambdaInvocation> for RuntimeApiClientService<S>
where
    S: Service<LambdaInvocation, Error = BoxError>,
    S::Future: Future<Output = Result<http::Request<Body>, BoxError>>,
{
    type Response = ();
    type Error = S::Error;
    type Future = RuntimeApiClientFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        let request_fut = self.inner.call(req);
        let client = self.client.clone();
        RuntimeApiClientFuture::First(request_fut, client)
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
            handle: self.handle.clone(),
            fallback_runtime: self.fallback_runtime.clone(),
        }
    }
}

// Despite this enum not being in mod.rs, it is `pub` and not marked as `#[non_exhaustive]`.
// This means that adding a new variant would be a breaking change for any downstream code
// that exhaustively matches on it. To consume the error response body without introducing
// a new variant, we instead spawn a background task via `tokio::task::spawn`.
#[pin_project(project = RuntimeApiClientFutureProj)]
pub enum RuntimeApiClientFuture<F> {
    /// The inner service is building the HTTP request to send to the Lambda Runtime API.
    First(#[pin] F, Arc<Client>),
    /// The HTTP request is in-flight; waiting for the Lambda Runtime API response.
    Second(#[pin] BoxFuture<'static, Result<http::Response<Incoming>, BoxError>>),
}

impl<F> Future for RuntimeApiClientFuture<F>
where
    F: Future<Output = Result<http::Request<Body>, BoxError>>,
{
    type Output = Result<(), BoxError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        // NOTE: We loop here to directly poll the second future once the first has finished.
        task::Poll::Ready(loop {
            match self.as_mut().project() {
                RuntimeApiClientFutureProj::First(fut, client) => match ready!(fut.poll(cx)) {
                    Ok(ok) => {
                        // NOTE: We use 'client.call_boxed' here to obtain a future with static
                        // lifetime. Otherwise, this future would need to be self-referential...
                        let next_fut = client
                            .call(ok)
                            .map_err(|err| {
                                log_or_print!(
                                    tracing: error!(error = ?err, "failed to send request to Lambda Runtime API"),
                                    fallback: eprintln!("Lambda Runtime API request failed: {err}")
                                );
                                err
                            })
                            .boxed();
                        self.set(RuntimeApiClientFuture::Second(next_fut));
                    }
                    Err(err) => break Err(err),
                },
                RuntimeApiClientFutureProj::Second(fut) => match ready!(fut.poll(cx)) {
                    Ok(resp) if !resp.status().is_success() => {
                        let status = resp.status();
                        // Spawn a background task to read and log the error body without
                        // blocking the current poll or requiring a new enum variant.
                        Handle::current().spawn(async move {
                            match resp.into_body().collect().await {
                                Ok(body) => {
                                    let body = body.to_bytes();
                                    log_or_print!(
                                        tracing: error!(status = %status, body = ?body, "Lambda Runtime API returned non-200 response"),
                                        fallback: eprintln!("Lambda Runtime API returned non-200 response: status={status}, body={body:?}")
                                    )
                                }
                                Err(e) => {
                                    log_or_print!(
                                        tracing: error!(status = %status, error = ?e, "Lambda Runtime API returned non-200 response (body unreadable)"),
                                        fallback: eprintln!("Lambda Runtime API returned non-200 response (body unreadable): status={status}, error={e:?}")
                                    )
                                }
                            }
                        });
                        break Ok(());
                    }
                    Ok(_) => break Ok(()),
                    Err(err) => break Err(err),
                },
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Context;
    use bytes::Bytes;
    use httpmock::prelude::*;
    use std::future;
    use tower::Service;

    fn make_invocation() -> LambdaInvocation {
        let (parts, _) = http::Response::new(()).into_parts();
        LambdaInvocation {
            parts,
            body: Bytes::new(),
            context: Context::default(),
        }
    }

    fn make_client(server: &MockServer) -> Arc<Client> {
        let uri = format!("http://{}:{}", server.host(), server.port()).parse().unwrap();
        Arc::new(Client::builder().with_endpoint(uri).build().unwrap())
    }

    fn mock_inner(
        req: http::Request<Body>,
    ) -> impl Service<
        LambdaInvocation,
        Response = http::Request<Body>,
        Error = BoxError,
        Future = impl Future<Output = Result<http::Request<Body>, BoxError>>,
    > {
        tower::service_fn(move |_inv: LambdaInvocation| {
            let req = http::Request::builder()
                .uri(req.uri().clone())
                .method(req.method().clone())
                .body(Body::empty())
                .unwrap();
            future::ready(Ok::<_, BoxError>(req))
        })
    }

    #[tokio::test]
    async fn test_2xx_response_succeeds() {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.any_request();
            then.status(200).body("ok");
        });

        let client = make_client(&server);
        let req = http::Request::builder().uri("/some/path").body(Body::empty()).unwrap();
        let mut svc = RuntimeApiClientService::new(mock_inner(req), client);

        let result = svc.call(make_invocation()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_invoke_timeout_410_is_logged_and_succeeds() {
        // Simulates the Lambda Runtime API returning 410 with an InvokeTimeout body,
        // e.g.: {"errorMessage":"Invoke timeout","errorType":"InvokeTimeout"}
        // The runtime should treat this as a non-fatal event: log it and return Ok(()).
        let server = MockServer::start();
        server.mock(|when, then| {
            when.any_request();
            then.status(410)
                .header("Content-Type", "application/json")
                .body(r#"{"errorMessage":"Invoke timeout","errorType":"InvokeTimeout"}"#);
        });

        let client = make_client(&server);
        let req = http::Request::builder().uri("/some/path").body(Body::empty()).unwrap();
        let mut svc = RuntimeApiClientService::new(mock_inner(req), client);

        // Non-2xx should still resolve Ok(()) — the error body is logged in a background task.
        let result = svc.call(make_invocation()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_inner_service_error_propagates() {
        let server = MockServer::start();
        let client = make_client(&server);

        let failing_inner = tower::service_fn(|_: LambdaInvocation| {
            future::ready(Err::<http::Request<Body>, BoxError>("inner error".into()))
        });
        let mut svc = RuntimeApiClientService::new(failing_inner, client);

        let result = svc.call(make_invocation()).await;
        assert!(result.is_err());
    }
}
