use crate::{
    layers::{CatchPanicService, RuntimeApiClientService, RuntimeApiResponseService},
    requests::{InitErrorRequest, IntoRequest, NextEventRequest, RestoreErrorRequest, RestoreNextRequest},
    snapstart::SnapStartResource,
    types::{invoke_request_id, IntoFunctionResponse, LambdaEvent},
    Config, Context, Diagnostic,
};
#[cfg(feature = "concurrency-tokio")]
use futures::stream::FuturesUnordered;
use http_body_util::BodyExt;
use lambda_runtime_api_client::{BoxError, PooledClient as ApiClient};
use serde::{Deserialize, Serialize};
#[cfg(feature = "concurrency-tokio")]
use std::fmt;
use std::{env, fmt::Debug, future::Future, sync::Arc};
use tokio_stream::{Stream, StreamExt};
use tower::{Layer, Service, ServiceExt};
use tracing::trace;
#[cfg(feature = "concurrency-tokio")]
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
    snapstart_resources: Vec<Arc<dyn SnapStartResource>>,
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
            ApiClient,
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
    ///
    ///
    /// # Panics
    ///
    /// This function panics if required Lambda environment variables are missing
    /// (`AWS_LAMBDA_FUNCTION_NAME`, `AWS_LAMBDA_FUNCTION_MEMORY_SIZE`,
    /// `AWS_LAMBDA_FUNCTION_VERSION`, `AWS_LAMBDA_RUNTIME_API`).
    pub fn new(handler: F) -> Self {
        trace!("Loading config from env");
        let config = Arc::new(Config::from_env());
        let concurrency_limit = max_concurrency_from_env().unwrap_or(1).max(1);
        // Strategy: allocate all worker tasks up-front, so size the client pool to match.
        let pool_size = concurrency_limit as usize;
        let client = Arc::new(ApiClient::builder().with_pool_size(pool_size).build());
        Self {
            service: wrap_handler(handler, client.clone()),
            config,
            client,
            concurrency_limit,
            snapstart_resources: Vec::new(),
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
            snapstart_resources: self.snapstart_resources,
        }
    }
}

impl<S> Runtime<S> {
    /// Returns `true` if the current execution environment has SnapStart enabled.
    pub fn is_snapstart(&self) -> bool {
        is_snapstart_env()
    }

    /// Register a [`SnapStartResource`] whose `before_snapshot`/`after_restore`
    /// hooks should run around the SnapStart snapshot/restore boundary.
    ///
    /// Register resources in dependency order — **foundations first** (e.g.
    /// credentials before the pool that depends on them). The runtime runs
    /// `before_snapshot` in reverse registration order (LIFO) and `after_restore`
    /// in registration order (FIFO), so teardown and rebuild both happen in the
    /// correct relative order. See the [`snapstart`](crate::snapstart) module
    /// docs for details.
    ///
    /// When SnapStart is not enabled, registered resources are never invoked and
    /// add no runtime overhead beyond the cost of holding the `Arc`.
    ///
    /// # Example
    /// ```no_run
    /// use lambda_runtime::{Error, LambdaEvent, Runtime, SnapStartResource};
    /// use std::sync::Arc;
    /// use serde_json::Value;
    /// use tower::service_fn;
    ///
    /// // Uses the default no-op hooks; override before_snapshot/after_restore as needed.
    /// struct Pool;
    /// impl SnapStartResource for Pool {}
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    ///     let pool = Arc::new(Pool);
    ///     let runtime = Runtime::new(service_fn(handler))
    ///         .register_snapstart_resource(pool.clone());
    ///     runtime.run().await
    /// }
    ///
    /// async fn handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    ///     Ok(event.payload)
    /// }
    /// ```
    pub fn register_snapstart_resource(mut self, resource: Arc<dyn SnapStartResource>) -> Self {
        self.snapstart_resources.push(resource);
        self
    }
}

#[cfg(feature = "concurrency-tokio")]
impl<S> Runtime<S>
where
    S: Service<LambdaInvocation, Response = (), Error = BoxError> + Clone + Send + 'static,
    S::Future: Send,
{
    /// Start the runtime and begin polling for events on the Lambda Runtime API,
    /// in a mode that is compatible with Lambda Managed Instances.
    ///
    /// When `AWS_LAMBDA_MAX_CONCURRENCY` is set to a value greater than 1, this
    /// spawns multiple tokio worker tasks to handle concurrent invocations. When the
    /// environment variable is unset or `<= 1`, it falls back to sequential
    /// behavior, so the same handler can run on both classic Lambda and Lambda
    /// Managed Instances.
    ///
    /// # Panics
    ///
    /// This function panics if called outside of a Tokio runtime.
    #[cfg_attr(docsrs, doc(cfg(feature = "concurrency-tokio")))]
    pub async fn run_concurrent(self) -> Result<(), BoxError> {
        if tokio::runtime::Handle::try_current().is_err() {
            panic!("`run_concurrent` must be called from within a Tokio runtime");
        }

        snapstart_lifecycle(&self.client, &self.snapstart_resources).await?;

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

#[cfg(feature = "concurrency-tokio")]
#[derive(Debug)]
enum WorkerError {
    CleanExit(tokio::task::Id),
    Failure(tokio::task::Id, BoxError),
}

#[cfg(feature = "concurrency-tokio")]
#[derive(Debug)]
struct ConcurrentWorkerErrors {
    errors: Vec<WorkerError>,
}

#[cfg(feature = "concurrency-tokio")]
#[derive(Serialize)]
struct ConcurrentWorkerErrorsPayload<'a> {
    message: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    clean: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    failures: Vec<WorkerFailurePayload>,
}

#[cfg(feature = "concurrency-tokio")]
#[derive(Serialize)]
struct WorkerFailurePayload {
    id: String,
    err: String,
}

#[cfg(feature = "concurrency-tokio")]
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

#[cfg(feature = "concurrency-tokio")]
impl std::error::Error for ConcurrentWorkerErrors {}

impl<S> Runtime<S>
where
    S: Service<LambdaInvocation, Response = (), Error = BoxError>,
{
    /// Start the runtime and begin polling for events on the Lambda Runtime API.
    ///
    /// The runtime will process requests sequentially.
    ///
    /// # Managed concurrency
    /// If `AWS_LAMBDA_MAX_CONCURRENCY` is set, a warning is logged.
    /// If your handler can satisfy `Clone + Send + 'static`,
    /// prefer [`Runtime::run_concurrent`] (requires the `concurrency-tokio` feature),
    /// which honors managed concurrency and falls back to sequential behavior when
    /// unset.
    pub async fn run(self) -> Result<(), BoxError> {
        if let Some(raw) = concurrency_env_value() {
            log_or_print!(
                tracing: tracing::warn!(
                    "AWS_LAMBDA_MAX_CONCURRENCY is set to '{raw}', but the concurrency-tokio feature is not enabled; running sequentially",
                ),
                fallback: eprintln!("AWS_LAMBDA_MAX_CONCURRENCY is set to '{raw}', but the concurrency-tokio feature is not enabled; running sequentially")
            );
        }
        snapstart_lifecycle(&self.client, &self.snapstart_resources).await?;
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

/// Returns `true` if the current execution environment has SnapStart enabled.
fn is_snapstart_env() -> bool {
    env::var("AWS_LAMBDA_INITIALIZATION_TYPE").as_deref() == Ok("snap-start")
}

/// Runs the SnapStart restore lifecycle when SnapStart is enabled, and is a
/// no-op otherwise.
///
/// The sequence is: run `before_snapshot` hooks (LIFO) → call `/restore/next`
/// (blocks until the VM is restored) → reset the internal RAPID connection pool
/// → run `after_restore` hooks (FIFO). A failure in the before-snapshot phase
/// (a hook or `/restore/next`) is reported to Lambda via `/init/error`; an
/// `after_restore` failure is reported via `/restore/error`. Either way the
/// error is returned so the runtime exits cleanly (no `process::exit`, so
/// graceful-shutdown handlers still run).
///
/// This is a free function taking only the fields it needs (rather than `&self`)
/// so it does not borrow the handler `S` across an `.await`, which would require
/// `Runtime<S>: Sync` in the by-value `run`/`run_concurrent` methods.
async fn snapstart_lifecycle(
    client: &Arc<ApiClient>,
    resources: &[Arc<dyn SnapStartResource>],
) -> Result<(), BoxError> {
    if !is_snapstart_env() {
        return Ok(());
    }
    run_restore_lifecycle(client, resources).await
}

/// Performs the restore lifecycle unconditionally (the env gate lives in the
/// caller, [`snapstart_lifecycle`]). Split out so it can be tested directly
/// without mutating the process-global `AWS_LAMBDA_INITIALIZATION_TYPE`.
async fn run_restore_lifecycle(
    client: &Arc<ApiClient>,
    resources: &[Arc<dyn SnapStartResource>],
) -> Result<(), BoxError> {
    // before_snapshot runs in REVERSE registration order (LIFO): tear down
    // dependents before their foundations. A failure here happens during init,
    // so report it to /init/error.
    for resource in resources.iter().rev() {
        if let Err(e) = resource.before_snapshot().await {
            return Err(report_init_error(client, e).await);
        }
    }

    // Signal init complete and block until the VM is restored from snapshot.
    let req = RestoreNextRequest.into_req()?;
    let resp = match client.call(req).await {
        Ok(resp) => resp,
        // A transport failure calling /restore/next is still an init-phase error.
        Err(e) => return Err(report_init_error(client, e).await),
    };
    // `call()` only surfaces transport errors; a non-2xx status from RAPID still
    // resolves to `Ok`. Treat a failed `/restore/next` as fatal rather than
    // silently proceeding into the invocation loop with an un-restored VM. This
    // still happens during init, so report it to /init/error.
    let status = resp.status();
    if !status.is_success() {
        let err: BoxError = format!("/restore/next returned a non-success status: {status}").into();
        return Err(report_init_error(client, err).await);
    }
    // Stale connections to RAPID won't survive the snapshot; rebuild the pool.
    client.reset_pool();

    // after_restore runs in registration order (FIFO): rebuild foundations
    // before the dependents that need them. A failure here is reported to
    // /restore/error and propagated so the runtime exits.
    for resource in resources {
        if let Err(e) = resource.after_restore().await {
            return Err(report_restore_error(client, e).await);
        }
    }

    Ok(())
}

/// Reports an init-phase error (a `before_snapshot` hook or `/restore/next`) to
/// Lambda via `/init/error`, then returns the original error so the caller can
/// propagate it and exit the runtime.
async fn report_init_error(client: &Arc<ApiClient>, err: BoxError) -> BoxError {
    report_diagnostic(client, err, ReportKind::Init).await
}

/// Reports an after-restore error to Lambda via `/restore/error`, then returns
/// the original error so the caller can propagate it and exit the runtime.
async fn report_restore_error(client: &Arc<ApiClient>, err: BoxError) -> BoxError {
    report_diagnostic(client, err, ReportKind::Restore).await
}

enum ReportKind {
    Init,
    Restore,
}

/// Posts a `Diagnostic` (built from the error) to the appropriate RAPID error
/// endpoint, then returns the **original** `BoxError` so the caller propagates it
/// with its concrete type and `source()` chain intact. Failures to build or send
/// the report are logged (never swallowed silently) but do not mask the error.
async fn report_diagnostic(client: &Arc<ApiClient>, err: BoxError, kind: ReportKind) -> BoxError {
    // Build the diagnostic from a reference so we can return `err` untouched.
    let diagnostic = Diagnostic {
        error_type: crate::diagnostic::type_name_of_val(&err),
        error_message: err.to_string(),
    };

    let (endpoint, req) = match kind {
        ReportKind::Init => ("/init/error", InitErrorRequest { diagnostic }.into_req()),
        ReportKind::Restore => ("/restore/error", RestoreErrorRequest { diagnostic }.into_req()),
    };

    match req {
        Ok(req) => {
            if let Err(e) = client.call(req).await {
                log_or_print!(
                    tracing: tracing::error!(error = ?e, "failed to report SnapStart error to {endpoint}"),
                    fallback: eprintln!("failed to report SnapStart error to {endpoint}: {e:?}")
                );
            }
        }
        Err(e) => {
            log_or_print!(
                tracing: tracing::error!(error = ?e, "failed to build SnapStart {endpoint} request"),
                fallback: eprintln!("failed to build SnapStart {endpoint} request: {e:?}")
            );
        }
    }

    err
}

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
    ApiClient,
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
#[cfg(feature = "concurrency-tokio")]
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

#[cfg(feature = "concurrency-tokio")]
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
        BoxFuture, Config, Diagnostic, Error, Runtime,
    };
    use bytes::Bytes;
    use http::{HeaderValue, Method, Request, Response, StatusCode};
    use http_body_util::{BodyExt, Full};
    use httpmock::prelude::*;

    use hyper::{body::Incoming, service::service_fn};
    use hyper_util::{
        rt::{tokio::TokioIo, TokioExecutor},
        server::conn::auto::Builder as ServerBuilder,
    };
    use lambda_runtime_api_client::PooledClient as Client;
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
        let client = Client::builder().with_endpoint(base).build();

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
        let client = Client::builder().with_endpoint(base).build();

        let req = EventCompletionRequest::new(
            "156cb537-e2d4-11e8-9b34-d36013741fb9",
            Option::Some("invocation_id"),
            "{}",
        );
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
        let client = Client::builder().with_endpoint(base).build();

        let req = EventErrorRequest {
            request_id: "156cb537-e2d4-11e8-9b34-d36013741fb9",
            invocation_id: Option::Some("invocation_id"),
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
        let client = Client::builder().with_endpoint(base).build();

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
            snapstart_resources: Vec::new(),
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
        let client = Client::builder().with_endpoint(base).build();

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
            snapstart_resources: Vec::new(),
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

    #[cfg(feature = "concurrency-tokio")]
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
        let client = Arc::new(Client::builder().with_endpoint(base).build());
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
            snapstart_resources: Vec::new(),
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

    #[cfg(feature = "concurrency-tokio")]
    // Must be current-thread (the default) so the thread-local tracing
    // subscriber set via `set_default` propagates to spawned tasks.
    #[tokio::test]
    async fn test_concurrent_structured_logging_isolation() -> Result<(), Error> {
        use std::collections::HashSet;
        use tracing::info;
        use tracing_capture::{CaptureLayer, SharedStorage};
        use tracing_subscriber::layer::SubscriberExt;

        let storage = SharedStorage::default();
        let subscriber = tracing_subscriber::registry().with(CaptureLayer::new(&storage));
        let _guard = tracing::subscriber::set_default(subscriber);

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
        let client = Arc::new(Client::builder().with_endpoint(base).build());

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
            snapstart_resources: Vec::new(),
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

    /// Records the order in which `before_snapshot`/`after_restore` fire across
    /// resources, so tests can assert LIFO/FIFO ordering.
    struct OrderRecorder {
        label: &'static str,
        log: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl crate::SnapStartResource for OrderRecorder {
        fn before_snapshot(&self) -> BoxFuture<'_, Result<(), Error>> {
            let log = self.log.clone();
            let label = self.label;
            Box::pin(async move {
                log.lock().unwrap().push(format!("before:{label}"));
                Ok(())
            })
        }

        fn after_restore(&self) -> BoxFuture<'_, Result<(), Error>> {
            let log = self.log.clone();
            let label = self.label;
            Box::pin(async move {
                log.lock().unwrap().push(format!("after:{label}"));
                Ok(())
            })
        }
    }

    /// Which lifecycle phase a [`FailingResource`] should fail in.
    #[derive(Clone, Copy)]
    enum Phase {
        BeforeSnapshot,
        AfterRestore,
    }

    /// A resource that returns an error from the selected phase (and is a no-op
    /// in the other), used to exercise the error-reporting paths.
    struct FailingResource {
        phase: Phase,
    }

    impl crate::SnapStartResource for FailingResource {
        fn before_snapshot(&self) -> BoxFuture<'_, Result<(), Error>> {
            let fail = matches!(self.phase, Phase::BeforeSnapshot);
            Box::pin(async move {
                if fail {
                    Err("before_snapshot failed".into())
                } else {
                    Ok(())
                }
            })
        }

        fn after_restore(&self) -> BoxFuture<'_, Result<(), Error>> {
            let fail = matches!(self.phase, Phase::AfterRestore);
            Box::pin(async move {
                if fail {
                    Err("after_restore failed".into())
                } else {
                    Ok(())
                }
            })
        }
    }

    #[tokio::test]
    async fn test_snapstart_restore_lifecycle_calls_restore_next() -> Result<(), Error> {
        let server = MockServer::start();

        let restore_mock = server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/restore/next");
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Arc::new(Client::builder().with_endpoint(base).build());

        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        let resources: Vec<Arc<dyn crate::SnapStartResource>> = vec![Arc::new(OrderRecorder {
            label: "pool",
            log: log.clone(),
        })];

        // Exercise the restore lifecycle directly (no env mutation, no races with
        // other tests that call run()/run_concurrent()).
        super::run_restore_lifecycle(&client, &resources).await?;

        restore_mock.assert_async().await;
        let recorded = log.lock().unwrap().clone();
        assert_eq!(recorded, vec!["before:pool", "after:pool"]);
        Ok(())
    }

    #[tokio::test]
    async fn test_snapstart_resource_ordering_is_lifo_then_fifo() -> Result<(), Error> {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/restore/next");
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Arc::new(Client::builder().with_endpoint(base).build());

        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        // Registration order: credentials → pool → cache (foundations first).
        let resources: Vec<Arc<dyn crate::SnapStartResource>> = vec![
            Arc::new(OrderRecorder {
                label: "credentials",
                log: log.clone(),
            }),
            Arc::new(OrderRecorder {
                label: "pool",
                log: log.clone(),
            }),
            Arc::new(OrderRecorder {
                label: "cache",
                log: log.clone(),
            }),
        ];

        super::run_restore_lifecycle(&client, &resources).await?;

        let recorded = log.lock().unwrap().clone();
        assert_eq!(
            recorded,
            vec![
                // before_snapshot: LIFO (reverse registration)
                "before:cache",
                "before:pool",
                "before:credentials",
                // after_restore: FIFO (registration order)
                "after:credentials",
                "after:pool",
                "after:cache",
            ]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_snapstart_restore_next_non_success_is_fatal() -> Result<(), Error> {
        let server = MockServer::start();
        server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/restore/next");
            then.status(500).body("");
        });
        let init_err_mock = server.mock(|when, then| {
            when.method(POST).path("/2018-06-01/runtime/init/error");
            then.status(202).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Arc::new(Client::builder().with_endpoint(base).build());

        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        let resources: Vec<Arc<dyn crate::SnapStartResource>> = vec![Arc::new(OrderRecorder {
            label: "pool",
            log: log.clone(),
        })];

        let result = super::run_restore_lifecycle(&client, &resources).await;

        // A non-2xx /restore/next must surface as an error rather than silently
        // continuing into the invocation loop with an un-restored VM, and it must
        // be reported to /init/error.
        assert!(result.is_err(), "expected non-success /restore/next to be fatal");
        init_err_mock.assert_async().await;
        // before_snapshot ran; after_restore must NOT run when the restore call fails.
        let recorded = log.lock().unwrap().clone();
        assert_eq!(recorded, vec!["before:pool"]);
        Ok(())
    }

    #[tokio::test]
    async fn test_snapstart_lifecycle_is_noop_when_not_snapstart() -> Result<(), Error> {
        // Only meaningful when the SnapStart env var is absent. Never mutate the
        // global env (would race with parallel tests); skip if it's set.
        if env::var("AWS_LAMBDA_INITIALIZATION_TYPE").is_ok() {
            return Ok(());
        }

        let server = MockServer::start();
        let restore_mock = server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/restore/next");
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Arc::new(Client::builder().with_endpoint(base).build());

        let log = Arc::new(std::sync::Mutex::new(Vec::new()));
        let resources: Vec<Arc<dyn crate::SnapStartResource>> = vec![Arc::new(OrderRecorder {
            label: "pool",
            log: log.clone(),
        })];

        // The env-gated entry point must do nothing when SnapStart is not enabled.
        super::snapstart_lifecycle(&client, &resources).await?;

        assert_eq!(
            restore_mock.calls(),
            0,
            "/restore/next must not be called when not snap-start"
        );
        assert!(
            log.lock().unwrap().is_empty(),
            "no hooks should fire when not snap-start"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_before_snapshot_failure_reports_init_error() -> Result<(), Error> {
        let server = MockServer::start();
        let init_err_mock = server.mock(|when, then| {
            when.method(POST).path("/2018-06-01/runtime/init/error");
            then.status(202).body("");
        });
        let restore_mock = server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/restore/next");
            then.status(200).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Arc::new(Client::builder().with_endpoint(base).build());

        let resources: Vec<Arc<dyn crate::SnapStartResource>> = vec![Arc::new(FailingResource {
            phase: Phase::BeforeSnapshot,
        })];

        let result = super::run_restore_lifecycle(&client, &resources).await;

        assert!(result.is_err(), "before_snapshot failure must propagate");
        init_err_mock.assert_async().await;
        // /restore/next must NOT be reached when before_snapshot fails.
        assert_eq!(restore_mock.calls(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_after_restore_failure_reports_restore_error() -> Result<(), Error> {
        let server = MockServer::start();
        let restore_mock = server.mock(|when, then| {
            when.method(GET).path("/2018-06-01/runtime/restore/next");
            then.status(200).body("");
        });
        let restore_err_mock = server.mock(|when, then| {
            when.method(POST).path("/2018-06-01/runtime/restore/error");
            then.status(202).body("");
        });

        let base = server.base_url().parse().expect("Invalid mock server Uri");
        let client = Arc::new(Client::builder().with_endpoint(base).build());

        let resources: Vec<Arc<dyn crate::SnapStartResource>> = vec![Arc::new(FailingResource {
            phase: Phase::AfterRestore,
        })];

        let result = super::run_restore_lifecycle(&client, &resources).await;

        // Now that the runtime no longer calls process::exit, the after_restore
        // failure path returns an error and is unit-testable.
        assert!(result.is_err(), "after_restore failure must propagate");
        restore_mock.assert_async().await;
        restore_err_mock.assert_async().await;
        Ok(())
    }
}
