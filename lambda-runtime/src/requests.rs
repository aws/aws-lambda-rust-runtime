use crate::{types::ToStreamErrorTrailer, Diagnostic, Error, FunctionResponse, IntoFunctionResponse};
use bytes::Bytes;
use http::{header::CONTENT_TYPE, Method, Request, Uri};
use lambda_runtime_api_client::{body::Body, build_request};
use serde::Serialize;
use std::{fmt::Debug, marker::PhantomData, str::FromStr};
use tokio_stream::{Stream, StreamExt};

pub(crate) trait IntoRequest {
    fn into_req(self) -> Result<Request<Body>, Error>;
}

// /runtime/invocation/next
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct NextEventRequest;

impl IntoRequest for NextEventRequest {
    fn into_req(self) -> Result<Request<Body>, Error> {
        let req = build_request()
            .method(Method::GET)
            .uri(Uri::from_static("/2018-06-01/runtime/invocation/next"))
            .body(Default::default())?;
        Ok(req)
    }
}

// /runtime/invocation/{AwsRequestId}/response
pub(crate) struct EventCompletionRequest<'a, R, B, S, D, E>
where
    R: IntoFunctionResponse<B, S>,
    B: Serialize,
    S: Stream<Item = Result<D, E>> + Unpin + Send + 'static,
    D: Into<Bytes> + Send,
    E: Into<Error> + Send + Debug,
{
    pub(crate) request_id: &'a str,
    pub(crate) body: R,
    pub(crate) _unused_b: PhantomData<B>,
    pub(crate) _unused_s: PhantomData<S>,
}

impl<'a, R, B, D, E, S> EventCompletionRequest<'a, R, B, S, D, E>
where
    R: IntoFunctionResponse<B, S>,
    B: Serialize,
    S: Stream<Item = Result<D, E>> + Unpin + Send + 'static,
    D: Into<Bytes> + Send,
    E: Into<Error> + Send + Debug,
{
    /// Initialize a new EventCompletionRequest
    pub(crate) fn new(request_id: &'a str, body: R) -> EventCompletionRequest<'a, R, B, S, D, E> {
        EventCompletionRequest {
            request_id,
            body,
            _unused_b: PhantomData::<B>,
            _unused_s: PhantomData::<S>,
        }
    }
}

impl<R, B, S, D, E> IntoRequest for EventCompletionRequest<'_, R, B, S, D, E>
where
    R: IntoFunctionResponse<B, S>,
    B: Serialize,
    S: Stream<Item = Result<D, E>> + Unpin + Send + 'static,
    D: Into<Bytes> + Send,
    E: Into<Error> + Send + Debug,
{
    fn into_req(self) -> Result<Request<Body>, Error> {
        match self.body.into_response() {
            FunctionResponse::BufferedResponse(body) => {
                let uri = format!("/2018-06-01/runtime/invocation/{}/response", self.request_id);
                let uri = Uri::from_str(&uri)?;

                let body = serde_json::to_vec(&body)?;
                let body = Body::from(body);

                let req = build_request().method(Method::POST).uri(uri).body(body)?;
                Ok(req)
            }
            FunctionResponse::StreamingResponse(mut response) => {
                let uri = format!("/2018-06-01/runtime/invocation/{}/response", self.request_id);
                let uri = Uri::from_str(&uri)?;

                let mut builder = build_request().method(Method::POST).uri(uri);
                let req_headers = builder.headers_mut().unwrap();

                req_headers.insert("Transfer-Encoding", "chunked".parse()?);
                req_headers.insert("Lambda-Runtime-Function-Response-Mode", "streaming".parse()?);
                // Report midstream errors using error trailers.
                // See the details in Lambda Developer Doc: https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html#runtimes-custom-response-streaming
                req_headers.append("Trailer", "Lambda-Runtime-Function-Error-Type".parse()?);
                req_headers.append("Trailer", "Lambda-Runtime-Function-Error-Body".parse()?);
                req_headers.insert(
                    "Content-Type",
                    "application/vnd.awslambda.http-integration-response".parse()?,
                );

                // default Content-Type
                let preloud_headers = &mut response.metadata_prelude.headers;
                preloud_headers
                    .entry(CONTENT_TYPE)
                    .or_insert("application/octet-stream".parse()?);

                let metadata_prelude = serde_json::to_string(&response.metadata_prelude)?;

                tracing::trace!(?metadata_prelude);

                let (mut tx, rx) = Body::channel();

                tokio::spawn(async move {
                    if tx.send_data(metadata_prelude.into()).await.is_err() {
                        tracing::error!("Error sending metadata prelude, response channel closed");
                        return;
                    }

                    if tx.send_data("\u{0}".repeat(8).into()).await.is_err() {
                        tracing::error!("Error sending metadata prelude delimiter, response channel closed");
                        return;
                    }

                    while let Some(chunk) = response.stream.next().await {
                        let chunk = match chunk {
                            Ok(chunk) => chunk.into(),
                            Err(err) => err.into().to_tailer().into(),
                        };

                        if tx.send_data(chunk).await.is_err() {
                            tracing::error!("Error sending response body chunk, response channel closed");
                            return;
                        }
                    }
                });

                let req = builder.body(rx)?;
                Ok(req)
            }
        }
    }
}

#[test]
fn test_event_completion_request() {
    let req = EventCompletionRequest::new("id", "hello, world!");
    let req = req.into_req().unwrap();
    let expected = Uri::from_static("/2018-06-01/runtime/invocation/id/response");
    assert_eq!(req.method(), Method::POST);
    assert_eq!(req.uri(), &expected);
    assert!(match req.headers().get("User-Agent") {
        Some(header) => header.to_str().unwrap().starts_with("aws-lambda-rust/"),
        None => false,
    });
}

// /runtime/invocation/{AwsRequestId}/error
pub(crate) struct EventErrorRequest<'a> {
    pub(crate) request_id: &'a str,
    pub(crate) diagnostic: Diagnostic,
}

impl<'a> EventErrorRequest<'a> {
    pub(crate) fn new(request_id: &'a str, diagnostic: impl Into<Diagnostic>) -> EventErrorRequest<'a> {
        EventErrorRequest {
            request_id,
            diagnostic: diagnostic.into(),
        }
    }
}

impl IntoRequest for EventErrorRequest<'_> {
    fn into_req(self) -> Result<Request<Body>, Error> {
        let uri = format!("/2018-06-01/runtime/invocation/{}/error", self.request_id);
        let uri = Uri::from_str(&uri)?;
        let body = serde_json::to_vec(&self.diagnostic)?;
        let body = Body::from(body);

        let req = build_request()
            .method(Method::POST)
            .uri(uri)
            .header("lambda-runtime-function-error-type", "unhandled")
            .body(body)?;
        Ok(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_event_request() {
        let req = NextEventRequest;
        let req = req.into_req().unwrap();
        assert_eq!(req.method(), Method::GET);
        assert_eq!(req.uri(), &Uri::from_static("/2018-06-01/runtime/invocation/next"));
        assert!(match req.headers().get("User-Agent") {
            Some(header) => header.to_str().unwrap().starts_with("aws-lambda-rust/"),
            None => false,
        });
    }

    #[test]
    fn test_event_error_request() {
        let req = EventErrorRequest {
            request_id: "id",
            diagnostic: Diagnostic {
                error_type: "InvalidEventDataError".into(),
                error_message: "Error parsing event data".into(),
            },
        };
        let req = req.into_req().unwrap();
        let expected = Uri::from_static("/2018-06-01/runtime/invocation/id/error");
        assert_eq!(req.method(), Method::POST);
        assert_eq!(req.uri(), &expected);
        assert!(match req.headers().get("User-Agent") {
            Some(header) => header.to_str().unwrap().starts_with("aws-lambda-rust/"),
            None => false,
        });
    }

    #[test]
    #[cfg(tokio_unstable)]
    fn streaming_send_data_error_is_ignored() {
        use crate::StreamResponse;

        let runtime = tokio::runtime::Builder::new_current_thread()
            .unhandled_panic(tokio::runtime::UnhandledPanic::ShutdownRuntime)
            .enable_all()
            .build()
            .unwrap();

        // We don't want to use a global panic hook to avoid shared state across tests, but we need to know
        // if a background task panics. We accomplish this by using UnhandledPanic::ShutdownRuntime
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runtime.block_on(async {
                let stream = tokio_stream::iter(vec![Ok::<Bytes, Error>(Bytes::from_static(b"chunk"))]);

                let stream_response: StreamResponse<_> = stream.into();
                let response = FunctionResponse::StreamingResponse(stream_response);

                let req: EventCompletionRequest<'_, _, (), _, _, _> = EventCompletionRequest::new("id", response);

                let http_req = req.into_req().expect("into_req should succeed");

                // immediate drop simulates client disconnection
                drop(http_req);

                // give the spawned task time to complete
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            })
        }));

        assert!(
            result.is_ok(),
            "spawned task panicked - send_data errors should be ignored"
        );
    }
}
