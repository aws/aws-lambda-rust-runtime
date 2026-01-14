use crate::{
    layers::{CatchPanicService, RuntimeApiClientService, RuntimeApiResponseService},
    requests::{IntoRequest, NextEventRequest},
    types::{invoke_request_id, IntoFunctionResponse, LambdaEvent},
    Config, Context, Diagnostic,
};
#[cfg(feature = "experimental-concurrency")]
use futures::stream::FuturesUnordered;
use http_body_util::BodyExt;
use lambda_runtime_api_client::{BoxError, Client as ApiClient};
use serde::{Deserialize, Serialize};
#[cfg(feature = "experimental-concurrency")]
use std::fmt;
use std::{env, fmt::Debug, future::Future, io, sync::Arc};
use tokio_stream::{Stream, StreamExt};
use tower::{Layer, Service, ServiceExt};
use tracing::trace;
#[cfg(feature = "experimental-concurrency")]
use tracing::{debug, error, info_span, warn, Instrument};

/* ----------------------------------------- INVOCATION ---------------------------------------- */

/// A simple container that provides information about a single invocation of a Lambda function.
pub struct LambdaInvocation {
    /// The header of the request sent to invoke the Lambda function.
    pub parts: http::response::Parts,
    /// The body of the request sent to invoke the Lambda function.
    pub body: bytes::Bytes,
    /// The context of the Lambda invocation.
    pub context: Context,
}

/* ------------------------------------------ RUNTIME ------------------------------------------ */

/// Lambda runtime executing a handler function on incoming requests.
///
/// Middleware can be added to a runtime using the [Runtime::layer] method in order to execute
/// logic prior to processing the incoming request and/or after the response has been sent back
/// to the Lambda Runtime API.
///
/// # Example
/// ```no_run
/// use lambda_runtime::{Error, LambdaEvent, Runtime};
/// use serde_json::Value;
/// use tower::service_fn;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let func = service_fn(func);
///     Runtime::new(func).run().await?;
///     Ok(())
/// }
///
/// async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
///     Ok(event.payload)
/// }
/// ````
pub struct Runtime<S> {
    service: S,
    config: Arc<Config>,
    client: Arc<ApiClient>,
    concurrency_limit: u32,
}

impl<F, EventPayload, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError>
    Runtime<
        RuntimeApiClientService<
            RuntimeApiResponseService<
                CatchPanicService<'_, F>,
                EventPayload,
                Response,
                BufferedResponse,
                StreamingResponse,
                StreamItem,
                StreamError,
            >,
        >,
    >
where
    F: Service<LambdaEvent<EventPayload>, Response = Response>,
    F::Future: Future<Output = Result<Response, F::Error>>,
    F::Error: Into<Diagnostic> + Debug,
    EventPayload: for<'de> Deserialize<'de>,
    Response: IntoFunctionResponse<BufferedResponse, StreamingResponse>,
    BufferedResponse: Serialize,
    StreamingResponse: Stream<Item = Result<StreamItem, StreamError>> + Unpin + Send + 'static,
    StreamItem: Into<bytes::Bytes> + Send,
    StreamError: Into<BoxError> + Send + Debug,
{
    /// Create a new runtime that executes the provided handler for incoming requests.
    ///
    /// In order to start the runtime and poll for events on the [Lambda Runtime
    /// APIs](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html), you must call
    /// [Runtime::run].
    ///
    /// Note that manually creating a [Runtime] does not add tracing to the executed handler
    /// as is done by [super::run]. If you want to add the default tracing functionality, call
    /// [Runtime::layer] with a [super::layers::TracingLayer].
    pub fn new(handler: F) -> Self {
        trace!("Loading config from env");
        let config = Arc::new(Config::from_env());
        let concurrency_limit = max_concurrency_from_env().unwrap_or(1).max(1);
        // Strategy: allocate all worker tasks up-front, so size the client pool to match.
        let pool_size = concurrency_limit as usize;
        let client = Arc::new(
            ApiClient::builder()
                .with_pool_size(pool_size)
                .build()
                .expect("Unable to create a runtime client"),
        );
        Self {
            service: wrap_handler(handler, client.clone()),
            config,
            client,
            concurrency_limit,
        }
    }
}

impl<S> Runtime<S> {
    /// Add a new layer to this runtime. For an incoming request, this layer will be executed
    /// before any layer that has been added prior.
    ///
    /// # Example
    /// ```no_run
    /// use lambda_runtime::{layers, Error, LambdaEvent, Runtime};
    /// use serde_json::Value;
    /// use tower::service_fn;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    ///     let runtime = Runtime::new(service_fn(echo)).layer(
    ///         layers::TracingLayer::new()
    ///     );
    ///     runtime.run().await?;
    ///     Ok(())
    /// }
    ///
    /// async fn echo(event: LambdaEvent<Value>) -> Result<Value, Error> {
    ///     Ok(event.payload)
    /// }
    /// ```
    pub fn layer<L>(self, layer: L) -> Runtime<L::Service>
    where
        L: Layer<S>,
        L::Service: Service<LambdaInvocation, Response = (), Error = BoxError>,
    {
        Runtime {
            client: self.client,
            config: self.config,
            service: layer.layer(self.service),
            concurrency_limit: self.concurrency_limit,
        }
    }
}

#[cfg(feature = "experimental-concurrency")]
impl<S> Runtime<S>
where
    S: Service<LambdaInvocation, Response = (), Error = BoxError> + Clone + Send + 'static,
    S::Future: Send,
{
    /// Start the runtime in concurrent mode when configured for Lambda managed-concurrency.
    ///
    /// If `AWS_LAMBDA_MAX_CONCURRENCY` is not set or is `<= 1`, this falls back to the
    /// sequential `run_with_incoming` loop so that the same handler can run on both
    /// classic Lambda and Lambda Managed Instances.
    #[cfg_attr(docsrs, doc(cfg(feature = "experimental-concurrency")))]
    pub async fn run_concurrent(self) -> Result<(), BoxError> {
        if self.concurrency_limit > 1 {
            trace!("Concurrent mode: _X_AMZN_TRACE_ID is not set; use context.xray_trace_id");
            Self::run_concurrent_inner(self.service, self.config, self.client, self.concurrency_limit).await
        } else {
            debug!(
                "Concurrent polling disabled (AWS_LAMBDA_MAX_CONCURRENCY unset or <= 1); falling back to sequential polling"
            );
            let incoming = incoming(&self.client);
            Self::run_with_incoming(self.service, self.config, incoming).await
        }
    }

    /// Concurrent processing using N independent long-poll loops (for Lambda managed-concurrency).
    async fn run_concurrent_inner(
        service: S,
        config: Arc<Config>,
        client: Arc<ApiClient>,
        concurrency_limit: u32,
    ) -> Result<(), BoxError> {
        let limit = concurrency_limit as usize;

        // Use FuturesUnordered so we can observe worker exits as they happen,
        // rather than waiting for all workers to finish (join_all).
        let mut workers: FuturesUnordered<tokio::task::JoinHandle<(tokio::task::Id, Result<(), BoxError>)>> =
            FuturesUnordered::new();
        let spawn_worker = |service: S, config: Arc<Config>, client: Arc<ApiClient>| {
            tokio::spawn(async move {
                let task_id = tokio::task::id();
                let result = concurrent_worker_loop(service, config, client).await;
                (task_id, result)
            })
        };
        // Spawn one worker per concurrency slot; the last uses the owned service to avoid an extra clone.
        for _ in 1..limit {
            workers.push(spawn_worker(service.clone(), config.clone(), client.clone()));
        }
        workers.push(spawn_worker(service, config, client));

        // Track worker exits across tasks. A single worker failing should not
        // terminate the whole runtime (LMI keeps running with the remaining
        // healthy workers). We only return an error once there are no workers
        // left (i.e., we cannot keep at least 1 worker alive).
        //
        // Note: Handler errors (Err returned from user code) do NOT trigger this.
        // They are reported to Lambda via /invocation/{id}/error and the worker
        // continues. This only captures unrecoverable runtime failures like
        // API client failures, runtime panics, etc.
        let mut errors: Vec<WorkerError> = Vec::new();
        let mut remaining_workers = limit;
        while let Some(result) = futures::StreamExt::next(&mut workers).await {
            remaining_workers = remaining_workers.saturating_sub(1);
            match result {
                Ok((task_id, Ok(()))) => {
                    // `concurrent_worker_loop` runs indefinitely, so an Ok return indicates
                    // an unexpected worker exit; we still decrement because the task is gone.
                    error!(
                        task_id = %task_id,
                        remaining_workers,
                        "Concurrent worker exited cleanly (unexpected - loop should run forever)"
                    );
                    errors.push(WorkerError::CleanExit(task_id));
                }
                Ok((task_id, Err(err))) => {
                    error!(
                        task_id = %task_id,
                        error = %err,
                        remaining_workers,
                        "Concurrent worker exited with error"
                    );
                    errors.push(WorkerError::Failure(task_id, err));
                }
                Err(join_err) => {
                    let task_id = join_err.id();
                    let err: BoxError = Box::new(join_err);
                    error!(
                        task_id = %task_id,
                        error = %err,
                        remaining_workers,
                        "Concurrent worker panicked"
                    );
                    errors.push(WorkerError::Failure(task_id, err));
                }
            }
        }

        match errors.len() {
            0 => Ok(()),
            _ => Err(Box::new(ConcurrentWorkerErrors { errors })),
        }
    }
}

#[cfg(feature = "experimental-concurrency")]
#[derive(Debug)]
enum WorkerError {
    CleanExit(tokio::task::Id),
    Failure(tokio::task::Id, BoxError),
}

#[cfg(feature = "experimental-concurrency")]
#[derive(Debug)]
struct ConcurrentWorkerErrors {
    errors: Vec<WorkerError>,
}

#[cfg(feature = "experimental-concurrency")]
#[derive(Serialize)]
struct ConcurrentWorkerErrorsPayload<'a> {
    message: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    clean: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    failures: Vec<WorkerFailurePayload>,
}

#[cfg(feature = "experimental-concurrency")]
#[derive(Serialize)]
struct WorkerFailurePayload {
    id: String,
    err: String,
}

#[cfg(feature = "experimental-concurrency")]
impl fmt::Display for ConcurrentWorkerErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut clean = Vec::new();
        let mut failures = Vec::new();
        for error in &self.errors {
            match error {
                WorkerError::CleanExit(task_id) => clean.push(task_id),
                WorkerError::Failure(task_id, err) => failures.push((task_id, err)),
            }
        }

        let clean_ids: Vec<String> = clean.iter().map(|task_id| task_id.to_string()).collect();
        let failure_entries: Vec<WorkerFailurePayload> = failures
            .iter()
            .map(|(task_id, err)| WorkerFailurePayload {
                id: task_id.to_string(),
                err: err.to_string(),
            })
            .collect();

        let message = if failures.is_empty() && !clean.is_empty() {
            "all concurrent workers exited cleanly (unexpected - loop should run forever)"
        } else {
            "concurrent workers exited unexpectedly"
        };

        let payload = ConcurrentWorkerErrorsPayload {
            message,
            clean: clean_ids,
            failures: failure_entries,
        };
        let json = serde_json::to_string(&payload).map_err(|_| fmt::Error)?;
        write!(f, "{json}")
    }
}

#[cfg(feature = "experimental-concurrency")]
impl std::error::Error for ConcurrentWorkerErrors {}

impl<S> Runtime<S>
where
    S: Service<LambdaInvocation, Response = (), Error = BoxError>,
{
    /// Start the runtime and begin polling for events on the Lambda Runtime API.
    ///
    /// If `AWS_LAMBDA_MAX_CONCURRENCY` is set, this returns an error because it does not enable
    /// concurrent polling. Enable the `experimental-concurrency` feature and use
    /// [`Runtime::run_concurrent`] instead.
    pub async fn run(self) -> Result<(), BoxError> {
        if let Some(raw) = concurrency_env_value() {
            return Err(Box::new(io::Error::other(format!(
                "AWS_LAMBDA_MAX_CONCURRENCY is set to '{raw}', but Runtime::run does not support concurrent polling; enable the experimental-concurrency feature and use Runtime::run_concurrent instead"
            ))));
        }
        let incoming = incoming(&self.client);
        Self::run_with_incoming(self.service, self.config, incoming).await
    }

    /// Internal utility function to start the runtime with a customized incoming stream.
    /// This implements the core of the [Runtime::run] method.
    pub(crate) async fn run_with_incoming(
        mut service: S,
        config: Arc<Config>,
        incoming: impl Stream<Item = Result<http::Response<hyper::body::Incoming>, BoxError>> + Send,
    ) -> Result<(), BoxError> {
        tokio::pin!(incoming);
        while let Some(next_event_response) = incoming.next().await {
            trace!("New event arrived (run loop)");
            let event = next_event_response?;
            process_invocation(&mut service, &config, event, true).await?;
        }
        Ok(())
    }
}

/* ------------------------------------------- UTILS ------------------------------------------- */

#[allow(clippy::type_complexity)]
fn wrap_handler<'a, F, EventPayload, Response, BufferedResponse, StreamingResponse, StreamItem, StreamError>(
    handler: F,
    client: Arc<ApiClient>,
) -> RuntimeApiClientService<
    RuntimeApiResponseService<
        CatchPanicService<'a, F>,
        EventPayload,
        Response,
        BufferedResponse,
        StreamingResponse,
        StreamItem,
        StreamError,
    >,
>
where
    F: Service<LambdaEvent<EventPayload>, Response = Response>,
    F::Future: Future<Output = Result<Response, F::Error>>,
    F::Error: Into<Diagnostic> + Debug,
    EventPayload: for<'de> Deserialize<'de>,
    Response: IntoFunctionResponse<BufferedResponse, StreamingResponse>,
    BufferedResponse: Serialize,
    StreamingResponse: Stream<Item = Result<StreamItem, StreamError>> + Unpin + Send + 'static,
    StreamItem: Into<bytes::Bytes> + Send,
    StreamError: Into<BoxError> + Send + Debug,
{
    let safe_service = CatchPanicService::new(handler);
    let response_service = RuntimeApiResponseService::new(safe_service);
    RuntimeApiClientService::new(response_service, client)
}

fn incoming(
    client: &ApiClient,
) -> impl Stream<Item = Result<http::Response<hyper::body::Incoming>, BoxError>> + Send + '_ {
    async_stream::stream! {
        loop {
            trace!("Waiting for next event (incoming loop)");
            let req = NextEventRequest.into_req().expect("Unable to construct request");
            let res = client.call(req).await;
            yield res;
        }
    }
}

/// Creates a future that polls the `/next` endpoint.
#[cfg(feature = "experimental-concurrency")]
async fn next_event_future(client: &ApiClient) -> Result<http::Response<hyper::body::Incoming>, BoxError> {
    let req = NextEventRequest.into_req()?;
    client.call(req).await
}

fn max_concurrency_from_env() -> Option<u32> {
    env::var("AWS_LAMBDA_MAX_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|&c| c > 0)
}

fn concurrency_env_value() -> Option<String> {
    env::var("AWS_LAMBDA_MAX_CONCURRENCY").ok()
}

#[cfg(feature = "experimental-concurrency")]
async fn concurrent_worker_loop<S>(mut service: S, config: Arc<Config>, client: Arc<ApiClient>) -> Result<(), BoxError>
where
    S: Service<LambdaInvocation, Response = (), Error = BoxError>,
    S::Future: Send,
{
    let task_id = tokio::task::id();
    let span = info_span!("worker", task_id = %task_id);
    loop {
        let event = match next_event_future(client.as_ref()).instrument(span.clone()).await {
            Ok(event) => event,
            Err(e) => {
                warn!(task_id = %task_id, error = %e, "Error polling /next, retrying");
                continue;
            }
        };

        process_invocation(&mut service, &config, event, false)
            .instrument(span.clone())
            .await?;
    }
}

async fn process_invocation<S>(
    service: &mut S,
    config: &Arc<Config>,
    event: http::Response<hyper::body::Incoming>,
    set_amzn_trace_env: bool,
) -> Result<(), BoxError>
where
    S: Service<LambdaInvocation, Response = (), Error = BoxError>,
{
    let (parts, incoming) = event.into_parts();

    #[cfg(debug_assertions)]
    if parts.status == http::StatusCode::NO_CONTENT {
        // Ignore the event if the status code is 204.
        // This is a way to keep the runtime alive when
        // there are no events pending to be processed.
        return Ok(());
    }

    // Build the invocation such that it can be sent to the service right away
    // when it is ready
    let body = incoming.collect().await?.to_bytes();
    let context = Context::new(invoke_request_id(&parts.headers)?, config.clone(), &parts.headers)?;
    let invocation = LambdaInvocation { parts, body, context };

    if set_amzn_trace_env {
        // Setup Amazon's default tracing data
        amzn_trace_env(&invocation.context);
    }

    // Wait for service to be ready
    let ready = service.ready().await?;

    // Once ready, call the service which will respond to the Lambda runtime API
    ready.call(invocation).await?;
    Ok(())
}

fn amzn_trace_env(ctx: &Context) {
    match &ctx.xray_trace_id {
        Some(trace_id) => env::set_var("_X_AMZN_TRACE_ID", trace_id),
        None => env::remove_var("_X_AMZN_TRACE_ID"),
    }
}

/* --------------------------------------------------------------------------------------------- */
/*                                             TESTS                                             */
/* --------------------------------------------------------------------------------------------- */

#[cfg(test)]
mod endpoint_tests {
    use super::{incoming, wrap_handler};
    use crate::{
        requests::{EventCompletionRequest, EventErrorRequest, IntoRequest, NextEventRequest},
        Config, Diagnostic, Error, Runtime,
    };
    use bytes::Bytes;
    use futures::future::BoxFuture;
    use http::{HeaderValue, Method, Request, Response, StatusCode};
    use http_body_util::{BodyExt, Full};
    use httpmock::prelude::*;

    use hyper::{body::Incoming, service::service_fn};
    use hyper_util::{
        rt::{tokio::TokioIo, TokioExecutor},
        server::conn::auto::Builder as ServerBuilder,
    };
    use lambda_runtime_api_client::Client;
    use std::{
        convert::Infallible,
        env,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };
    use tokio::{net::TcpListener, sync::Notify};
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_next_event() -> Result<(), Error> {
        let server = MockServer::start();
        let request_id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
        let deadline = "1542409706888";

        let mock = server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/invocation/next");
            then.status(200)
                .header("content-type", "application/json")
                .header("lambda-runtime-aws-request-id", request_id)
                .header("lambda-runtime-deadline-ms", deadline)
                .body("{}");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Client::builder().with_endpoint(base).build()?;

        let req = NextEventRequest.into_req()?;
        let rsp = client.call(req).await.expect("Unable to send request");

        mock.assert_async().await;
        assert_eq!(rsp.status(), StatusCode::OK);
        assert_eq!(
            rsp.headers()["lambda-runtime-aws-request-id"],
            &HeaderValue::from_static(request_id)
        );
        assert_eq!(
            rsp.headers()["lambda-runtime-deadline-ms"],
            &HeaderValue::from_static(deadline)
        );

        let body = rsp.into_body().collect().await?.to_bytes();
        assert_eq!("{}", std::str::from_utf8(&body)?);
        Ok(())
    }

    #[tokio::test]
    async fn test_ok_response() -> Result<(), Error> {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/2018-06-01/runtime/invocation/156cb537-e2d4-11e8-9b34-d36013741fb9/response")
                .body("\"{}\"");
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Client::builder().with_endpoint(base).build()?;

        let req = EventCompletionRequest::new("156cb537-e2d4-11e8-9b34-d36013741fb9", "{}");
        let req = req.into_req()?;

        let rsp = client.call(req).await?;

        mock.assert_async().await;
        assert_eq!(rsp.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn test_error_response() -> Result<(), Error> {
        let diagnostic = Diagnostic {
            error_type: "InvalidEventDataError".into(),
            error_message: "Error parsing event data".into(),
        };
        let body = serde_json::to_string(&diagnostic)?;

        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/2018-06-01/runtime/invocation/156cb537-e2d4-11e8-9b34-d36013741fb9/error")
                .header("lambda-runtime-function-error-type", "unhandled")
                .body(body);
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Client::builder().with_endpoint(base).build()?;

        let req = EventErrorRequest {
            request_id: "156cb537-e2d4-11e8-9b34-d36013741fb9",
            diagnostic,
        };
        let req = req.into_req()?;
        let rsp = client.call(req).await?;

        mock.assert_async().await;
        assert_eq!(rsp.status(), StatusCode::OK);
        Ok(())
    }

    #[tokio::test]
    async fn successful_end_to_end_run() -> Result<(), Error> {
        let server = MockServer::start();
        let request_id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
        let deadline = "1542409706888";

        let next_request = server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/invocation/next");
            then.status(200)
                .header("content-type", "application/json")
                .header("lambda-runtime-aws-request-id", request_id)
                .header("lambda-runtime-deadline-ms", deadline)
                .body("{}");
        });
        let next_response = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/2018-06-01/runtime/invocation/{request_id}/response"))
                .body("{}");
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Client::builder().with_endpoint(base).build()?;

        async fn func(event: crate::LambdaEvent<serde_json::Value>) -> Result<serde_json::Value, Error> {
            let (event, _) = event.into_parts();
            Ok(event)
        }
        let f = crate::service_fn(func);

        // set env vars needed to init Config if they are not already set in the environment
        if env::var("AWS_LAMBDA_RUNTIME_API").is_err() {
            env::set_var("AWS_LAMBDA_RUNTIME_API", server.base_url());
        }
        if env::var("AWS_LAMBDA_FUNCTION_NAME").is_err() {
            env::set_var("AWS_LAMBDA_FUNCTION_NAME", "test_fn");
        }
        if env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE").is_err() {
            env::set_var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE", "128");
        }
        if env::var("AWS_LAMBDA_FUNCTION_VERSION").is_err() {
            env::set_var("AWS_LAMBDA_FUNCTION_VERSION", "1");
        }
        if env::var("AWS_LAMBDA_LOG_STREAM_NAME").is_err() {
            env::set_var("AWS_LAMBDA_LOG_STREAM_NAME", "test_stream");
        }
        if env::var("AWS_LAMBDA_LOG_GROUP_NAME").is_err() {
            env::set_var("AWS_LAMBDA_LOG_GROUP_NAME", "test_log");
        }
        let config = Config::from_env();

        let client = Arc::new(client);
        let runtime = Runtime {
            client: client.clone(),
            config: Arc::new(config),
            service: wrap_handler(f, client),
            concurrency_limit: 1,
        };
        let client = &runtime.client;
        let incoming = incoming(client).take(1);
        Runtime::run_with_incoming(runtime.service, runtime.config, incoming).await?;

        next_request.assert_async().await;
        next_response.assert_async().await;
        Ok(())
    }

    async fn run_panicking_handler<F>(func: F) -> Result<(), Error>
    where
        F: FnMut(crate::LambdaEvent<serde_json::Value>) -> BoxFuture<'static, Result<serde_json::Value, Error>>
            + Send
            + 'static,
    {
        let server = MockServer::start();
        let request_id = "156cb537-e2d4-11e8-9b34-d36013741fb9";
        let deadline = "1542409706888";

        let next_request = server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/invocation/next");
            then.status(200)
                .header("content-type", "application/json")
                .header("lambda-runtime-aws-request-id", request_id)
                .header("lambda-runtime-deadline-ms", deadline)
                .body("{}");
        });

        let next_response = server.mock(|when, then| {
            when.method(POST)
                .path(format!("/2018-06-01/runtime/invocation/{request_id}/error"))
                .header("lambda-runtime-function-error-type", "unhandled");
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Client::builder().with_endpoint(base).build()?;

        let f = crate::service_fn(func);

        let config = Arc::new(Config {
            function_name: "test_fn".to_string(),
            memory: 128,
            version: "1".to_string(),
            log_stream: "test_stream".to_string(),
            log_group: "test_log".to_string(),
        });

        let client = Arc::new(client);
        let runtime = Runtime {
            client: client.clone(),
            config,
            service: wrap_handler(f, client),
            concurrency_limit: 1,
        };
        let client = &runtime.client;
        let incoming = incoming(client).take(1);
        Runtime::run_with_incoming(runtime.service, runtime.config, incoming).await?;

        next_request.assert_async().await;
        next_response.assert_async().await;
        Ok(())
    }

    #[tokio::test]
    async fn panic_in_async_run() -> Result<(), Error> {
        run_panicking_handler(|_| Box::pin(async { panic!("This is intentionally here") })).await
    }

    #[tokio::test]
    async fn panic_outside_async_run() -> Result<(), Error> {
        run_panicking_handler(|_| {
            panic!("This is intentionally here");
        })
        .await
    }

    #[cfg(feature = "experimental-concurrency")]
    #[tokio::test]
    async fn concurrent_worker_crash_does_not_stop_other_workers() -> Result<(), Error> {
        let next_calls = Arc::new(AtomicUsize::new(0));
        let response_calls = Arc::new(AtomicUsize::new(0));
        let first_error_served = Arc::new(Notify::new());

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base: http::Uri = format!("http://{addr}").parse().unwrap();

        let server_handle = {
            let next_calls = next_calls.clone();
            let response_calls = response_calls.clone();
            let first_error_served = first_error_served.clone();
            tokio::spawn(async move {
                loop {
                    let (tcp, _) = match listener.accept().await {
                        Ok(v) => v,
                        Err(_) => return,
                    };

                    let next_calls = next_calls.clone();
                    let response_calls = response_calls.clone();
                    let first_error_served = first_error_served.clone();
                    let service = service_fn(move |req: Request<Incoming>| {
                        let next_calls = next_calls.clone();
                        let response_calls = response_calls.clone();
                        let first_error_served = first_error_served.clone();
                        async move {
                            let (parts, body) = req.into_parts();
                            let method = parts.method;
                            let path = parts.uri.path().to_string();

                            if method == Method::POST {
                                // Drain request body to support keep-alive.
                                let _ = body.collect().await;
                            }

                            if method == Method::GET && path == "/2018-06-01/runtime/invocation/next" {
                                let call_index = next_calls.fetch_add(1, Ordering::SeqCst);
                                match call_index {
                                    // First worker errors (missing request id header).
                                    0 => {
                                        first_error_served.notify_one();
                                        let res = Response::builder()
                                            .status(StatusCode::OK)
                                            .header("lambda-runtime-deadline-ms", "1542409706888")
                                            .body(Full::new(Bytes::from_static(b"{}")))
                                            .unwrap();
                                        return Ok::<_, Infallible>(res);
                                    }
                                    // Second worker should keep running and process an invocation, even if another worker errors.
                                    1 => {
                                        first_error_served.notified().await;
                                        let res = Response::builder()
                                            .status(StatusCode::OK)
                                            .header("content-type", "application/json")
                                            .header("lambda-runtime-aws-request-id", "good-request")
                                            .header("lambda-runtime-deadline-ms", "1542409706888")
                                            .body(Full::new(Bytes::from_static(b"{}")))
                                            .unwrap();
                                        return Ok::<_, Infallible>(res);
                                    }
                                    // Finally, error the remaining worker so the runtime can terminate and the test can assert behavior.
                                    2 => {
                                        let res = Response::builder()
                                            .status(StatusCode::OK)
                                            .header("lambda-runtime-deadline-ms", "1542409706888")
                                            .body(Full::new(Bytes::from_static(b"{}")))
                                            .unwrap();
                                        return Ok::<_, Infallible>(res);
                                    }
                                    _ => {
                                        let res = Response::builder()
                                            .status(StatusCode::NO_CONTENT)
                                            .body(Full::new(Bytes::new()))
                                            .unwrap();
                                        return Ok::<_, Infallible>(res);
                                    }
                                }
                            }

                            if method == Method::POST && path.ends_with("/response") {
                                response_calls.fetch_add(1, Ordering::SeqCst);
                                let res = Response::builder()
                                    .status(StatusCode::OK)
                                    .body(Full::new(Bytes::new()))
                                    .unwrap();
                                return Ok::<_, Infallible>(res);
                            }

                            let res = Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Full::new(Bytes::new()))
                                .unwrap();
                            Ok::<_, Infallible>(res)
                        }
                    });

                    let io = TokioIo::new(tcp);
                    tokio::spawn(async move {
                        if let Err(err) = ServerBuilder::new(TokioExecutor::new())
                            .serve_connection(io, service)
                            .await
                        {
                            eprintln!("Error serving connection: {err:?}");
                        }
                    });
                }
            })
        };

        async fn func(event: crate::LambdaEvent<serde_json::Value>) -> Result<serde_json::Value, Error> {
            Ok(event.payload)
        }

        let handler = crate::service_fn(func);
        let client = Arc::new(Client::builder().with_endpoint(base).build()?);
        let runtime = Runtime {
            client: client.clone(),
            config: Arc::new(Config {
                function_name: "test_fn".to_string(),
                memory: 128,
                version: "1".to_string(),
                log_stream: "test_stream".to_string(),
                log_group: "test_log".to_string(),
            }),
            service: wrap_handler(handler, client),
            concurrency_limit: 2,
        };

        let res = tokio::time::timeout(Duration::from_secs(2), runtime.run_concurrent()).await;
        assert!(res.is_ok(), "run_concurrent timed out");
        assert!(
            res.unwrap().is_err(),
            "expected runtime to terminate once all workers crashed"
        );

        assert_eq!(
            response_calls.load(Ordering::SeqCst),
            1,
            "expected remaining worker to keep running after a worker crash"
        );

        server_handle.abort();
        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "experimental-concurrency")]
    async fn test_concurrent_structured_logging_isolation() -> Result<(), Error> {
        use std::collections::HashSet;
        use tracing::info;
        use tracing_capture::{CaptureLayer, SharedStorage};
        use tracing_subscriber::layer::SubscriberExt;

        let storage = SharedStorage::default();
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(&storage));
        tracing::subscriber::set_global_default(subscriber).unwrap();

        let request_count = Arc::new(AtomicUsize::new(0));
        let done = Arc::new(tokio::sync::Notify::new());
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let base: http::Uri = format!("http://{addr}").parse()?;

        let server_handle = {
            let request_count = request_count.clone();
            let done = done.clone();
            tokio::spawn(async move {
                loop {
                    let (tcp, _) = match listener.accept().await {
                        Ok(v) => v,
                        Err(_) => return,
                    };

                    let request_count = request_count.clone();
                    let done = done.clone();
                    let service = service_fn(move |req: Request<Incoming>| {
                        let request_count = request_count.clone();
                        let done = done.clone();
                        async move {
                            let (parts, body) = req.into_parts();
                            if parts.method == Method::POST {
                                let _ = body.collect().await;
                            }

                            if parts.method == Method::GET && parts.uri.path() == "/2018-06-01/runtime/invocation/next"
                            {
                                let count = request_count.fetch_add(1, Ordering::SeqCst);
                                if count < 300 {
                                    let request_id = format!("test-request-{}", count + 1);
                                    let res = Response::builder()
                                        .status(StatusCode::OK)
                                        .header("lambda-runtime-aws-request-id", &request_id)
                                        .header("lambda-runtime-deadline-ms", "9999999999999")
                                        .body(Full::new(Bytes::from_static(b"{}")))
                                        .unwrap();
                                    return Ok::<_, Infallible>(res);
                                } else {
                                    done.notify_one();
                                    let res = Response::builder()
                                        .status(StatusCode::NO_CONTENT)
                                        .body(Full::new(Bytes::new()))
                                        .unwrap();
                                    return Ok::<_, Infallible>(res);
                                }
                            }

                            if parts.method == Method::POST && parts.uri.path().contains("/response") {
                                let res = Response::builder()
                                    .status(StatusCode::OK)
                                    .body(Full::new(Bytes::new()))
                                    .unwrap();
                                return Ok::<_, Infallible>(res);
                            }

                            let res = Response::builder()
                                .status(StatusCode::NOT_FOUND)
                                .body(Full::new(Bytes::new()))
                                .unwrap();
                            Ok::<_, Infallible>(res)
                        }
                    });

                    let io = TokioIo::new(tcp);
                    tokio::spawn(async move {
                        let _ = ServerBuilder::new(TokioExecutor::new())
                            .serve_connection(io, service)
                            .await;
                    });
                }
            })
        };

        async fn test_handler(event: crate::LambdaEvent<serde_json::Value>) -> Result<(), Error> {
            let request_id = &event.context.request_id;
            info!(observed_request_id = request_id);
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(())
        }

        let handler = crate::service_fn(test_handler);
        let client = Arc::new(Client::builder().with_endpoint(base).build()?);

        // Add tracing layer to capture span fields
        use crate::layers::trace::TracingLayer;
        use tower::ServiceBuilder;
        let service = ServiceBuilder::new()
            .layer(TracingLayer::new())
            .service(wrap_handler(handler, client.clone()));

        let runtime = Runtime {
            client: client.clone(),
            config: Arc::new(Config {
                function_name: "test_fn".to_string(),
                memory: 128,
                version: "1".to_string(),
                log_stream: "test_stream".to_string(),
                log_group: "test_log".to_string(),
            }),
            service,
            concurrency_limit: 3,
        };

        let runtime_handle = tokio::spawn(async move { runtime.run_concurrent().await });

        done.notified().await;
        // Give handlers time to complete after server signals done
        tokio::time::sleep(Duration::from_millis(500)).await;

        runtime_handle.abort();
        server_handle.abort();

        let storage = storage.lock();
        let events: Vec<_> = storage
            .all_events()
            .filter(|e| e.value("observed_request_id").is_some())
            .collect();

        assert!(
            events.len() >= 300,
            "Should have at least 300 log entries, got {}",
            events.len()
        );

        let mut seen_ids = HashSet::new();
        for event in &events {
            let observed_id = event["observed_request_id"].as_str().unwrap();

            // Find the parent "Lambda runtime invoke" span and get its requestId
            let span_request_id = event
                .ancestors()
                .find(|s| s.metadata().name() == "Lambda runtime invoke")
                .and_then(|s| s.value("requestId"))
                .and_then(|v| v.as_str())
                .expect("Event should have a Lambda runtime invoke ancestor with requestId");

            assert!(
                observed_id.starts_with("test-request-"),
                "Request ID should match pattern: {}",
                observed_id
            );
            assert!(
                seen_ids.insert(observed_id.to_string()),
                "Request ID should be unique: {}",
                observed_id
            );

            // Verify span request ID matches logged request ID
            assert_eq!(
                observed_id, span_request_id,
                "Span request ID should match logged request ID: span={}, logged={}",
                span_request_id, observed_id
            );
        }

        Ok(())
    }
}
