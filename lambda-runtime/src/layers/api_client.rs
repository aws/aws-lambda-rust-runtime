use crate::LambdaInvocation;
use futures::{future::BoxFuture, ready, FutureExt, TryFutureExt};
use hyper::body::Incoming;
use lambda_runtime_api_client::{body::Body, BoxError, Client};
use pin_project::pin_project;
use std::{future::Future, pin::Pin, sync::Arc, task};
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
                    Err(err) => {
                        log_or_print!(
                            tracing: tracing::error!(error = ?err, "failed to build Lambda Runtime API request"),
                            fallback: eprintln!("failed to build Lambda Runtime API request: {err:?}")
                        );
                        break Err(err);
                    }
                },
                RuntimeApiClientFutureProj::Second(fut) => match ready!(fut.poll(cx)) {
                    Ok(resp) if !resp.status().is_success() => {
                        let status = resp.status();

                        // TODO
                        // we should consume the response body of the call in order to give a more specific message.
                        // https://github.com/aws/aws-lambda-rust-runtime/issues/1110

                        log_or_print!(
                            tracing: tracing::error!(status = %status, "Lambda Runtime API returned non-200 response"),
                            fallback: eprintln!("Lambda Runtime API returned non-200 response: status={status}")
                        );

                        // Adding more information on top of 410 Gone, to make it more clear since we cannot access the body of the message
                        if status == 410 {
                            log_or_print!(
                                tracing: tracing::error!("Lambda function timeout!"),
                                fallback: eprintln!("Lambda function timeout!")
                            );
                        }

                        // Return Ok to maintain existing contract - runtime continues despite API errors
                        break Ok(());
                    }
                    Ok(_) => break Ok(()),
                    Err(err) => {
                        log_or_print!(
                            tracing: tracing::error!(error = ?err, "Lambda Runtime API request failed"),
                            fallback: eprintln!("Lambda Runtime API request failed: {err:?}")
                        );
                        break Err(err);
                    }
                },
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::StatusCode;
    use http_body_util::Full;
    use hyper::body::Bytes;
    use lambda_runtime_api_client::body::Body;
    use std::convert::Infallible;
    use tokio::net::TcpListener;
    use tracing_test::traced_test;

    async fn start_mock_server(status: StatusCode) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}", addr);

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let io = hyper_util::rt::TokioIo::new(stream);

            let service = hyper::service::service_fn(move |_req| async move {
                Ok::<_, Infallible>(
                    http::Response::builder()
                        .status(status)
                        .body(Full::new(Bytes::from("test response")))
                        .unwrap(),
                )
            });

            let _ = hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
                .serve_connection(io, service)
                .await;
        });

        // Give the server a moment to start
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        url
    }

    #[tokio::test]
    #[traced_test]
    async fn test_successful_response() {
        let url = start_mock_server(StatusCode::OK).await;
        let client = Arc::new(
            lambda_runtime_api_client::Client::builder()
                .with_endpoint(url.parse().unwrap())
                .build()
                .unwrap(),
        );

        let request_fut =
            async { Ok::<_, BoxError>(http::Request::builder().uri("/test").body(Body::empty()).unwrap()) };

        let future = RuntimeApiClientFuture::First(request_fut, client);
        let result = future.await;

        assert!(result.is_ok());
        // No error logs should be present
        assert!(!logs_contain("Lambda Runtime API returned non-200 response"));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_410_timeout_error() {
        let url = start_mock_server(StatusCode::GONE).await;
        let client = Arc::new(
            lambda_runtime_api_client::Client::builder()
                .with_endpoint(url.parse().unwrap())
                .build()
                .unwrap(),
        );

        let request_fut =
            async { Ok::<_, BoxError>(http::Request::builder().uri("/test").body(Body::empty()).unwrap()) };

        let future = RuntimeApiClientFuture::First(request_fut, client);
        let result = future.await;

        // Returns Ok to maintain contract, but logs the error
        assert!(result.is_ok());

        // Verify the error was logged
        assert!(logs_contain("Lambda Runtime API returned non-200 response"));
        assert!(logs_contain("Lambda function timeout!"));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_500_error() {
        let url = start_mock_server(StatusCode::INTERNAL_SERVER_ERROR).await;
        let client = Arc::new(
            lambda_runtime_api_client::Client::builder()
                .with_endpoint(url.parse().unwrap())
                .build()
                .unwrap(),
        );

        let request_fut =
            async { Ok::<_, BoxError>(http::Request::builder().uri("/test").body(Body::empty()).unwrap()) };

        let future = RuntimeApiClientFuture::First(request_fut, client);
        let result = future.await;

        // Returns Ok to maintain contract, but logs the error
        assert!(result.is_ok());

        // Verify the error was logged with status code
        assert!(logs_contain("Lambda Runtime API returned non-200 response"));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_404_error() {
        let url = start_mock_server(StatusCode::NOT_FOUND).await;
        let client = Arc::new(
            lambda_runtime_api_client::Client::builder()
                .with_endpoint(url.parse().unwrap())
                .build()
                .unwrap(),
        );

        let request_fut =
            async { Ok::<_, BoxError>(http::Request::builder().uri("/test").body(Body::empty()).unwrap()) };

        let future = RuntimeApiClientFuture::First(request_fut, client);
        let result = future.await;

        // Returns Ok to maintain contract, but logs the error
        assert!(result.is_ok());

        // Verify the error was logged
        assert!(logs_contain("Lambda Runtime API returned non-200 response"));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_request_build_error() {
        let client = Arc::new(
            lambda_runtime_api_client::Client::builder()
                .with_endpoint("http://localhost:9001".parse().unwrap())
                .build()
                .unwrap(),
        );

        let request_fut = async { Err::<http::Request<Body>, BoxError>("Request build error".into()) };

        let future = RuntimeApiClientFuture::First(request_fut, client);
        let result = future.await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Request build error"));

        // Verify the error was logged
        assert!(logs_contain("failed to build Lambda Runtime API request"));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_network_error() {
        // Use an invalid endpoint that will fail to connect
        let client = Arc::new(
            lambda_runtime_api_client::Client::builder()
                .with_endpoint("http://127.0.0.1:1".parse().unwrap()) // Port 1 should be unreachable
                .build()
                .unwrap(),
        );

        let request_fut =
            async { Ok::<_, BoxError>(http::Request::builder().uri("/test").body(Body::empty()).unwrap()) };

        let future = RuntimeApiClientFuture::First(request_fut, client);
        let result = future.await;

        // Network errors should propagate as Err
        assert!(result.is_err());

        // Verify the error was logged
        assert!(logs_contain("Lambda Runtime API request failed"));
    }
}
