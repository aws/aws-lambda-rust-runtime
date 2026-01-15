use crate::LambdaInvocation;
use futures::{future::BoxFuture, ready, FutureExt, TryFutureExt};
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
        }
    }
}

#[pin_project(project = RuntimeApiClientFutureProj)]
pub enum RuntimeApiClientFuture<F> {
    First(#[pin] F, Arc<Client>),
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
                                error!(error = ?err, "failed to send request to Lambda Runtime API");
                                err
                            })
                            .boxed();
                        self.set(RuntimeApiClientFuture::Second(next_fut));
                    }
                    Err(err) => break Err(err),
                },
                RuntimeApiClientFutureProj::Second(fut) => {
                    let response = ready!(fut.poll(cx))?;
                    
                    // We need to consume the body to get trailers (they come after the body)
                    // For now, we'll check headers for the End-Of-Response trailer announcement
                    // and log a warning if timeout is indicated
                    // TODO: Properly consume body and check trailers for "End-Of-Response: timeout"
                    
                    // Check if response indicates a timeout in headers
                    // (Lambda may use headers or trailers depending on the scenario)
                    if let Some(status) = response.headers().get("Lambda-Runtime-Function-Response-Status") {
                        if status == "timeout" {
                            warn!("Lambda invocation timed out - response was not sent within the configured timeout");
                        }
                    }
                    
                    // Note: To properly check trailers, we would need to:
                    // 1. Collect the body: let collected = response.into_body().collect().await?;
                    // 2. Check trailers: if let Some(trailers) = collected.trailers() {
                    // 3. Look for: trailers.get("End-Of-Response") == Some("timeout")
                    // However, this would require making this future async and changing the architecture
                    
                    break Ok(());
                }
            }
        })
    }
}
