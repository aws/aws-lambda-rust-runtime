#![deny(clippy::all, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! The mechanism available for defining a Lambda function is as follows:
//!
//! Create a type that conforms to the [`tower::Service`] trait. This type can
//! then be passed to the the `lambda_runtime::run` function, which launches
//! and runs the Lambda runtime.
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::{self, Debug},
    future::Future,
    sync::Arc,
};
use tokio_stream::Stream;
use tower::util::ServiceFn;
pub use tower::{self, service_fn, Service};

/// Diagnostic utilities to convert Rust types into Lambda Error types.
pub mod diagnostic;
pub use diagnostic::Diagnostic;

mod deserializer;
/// Tower middleware to be applied to runtime invocations.
pub mod layers;
mod requests;
mod runtime;
/// Utilities for Lambda Streaming functions.
pub mod streaming;

/// Utilities to initialize and use `tracing` and `tracing-subscriber` in Lambda Functions.
#[cfg(feature = "tracing")]
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
pub use lambda_runtime_api_client::tracing;

/// Types available to a Lambda function.
mod types;

use requests::EventErrorRequest;
pub use runtime::{LambdaInvocation, Runtime};
pub use types::{Context, FunctionResponse, IntoFunctionResponse, LambdaEvent, MetadataPrelude, StreamResponse};

/// Error type that lambdas may result in
pub type Error = lambda_runtime_api_client::BoxError;

/// Configuration derived from environment variables.
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The name of the function.
    pub function_name: String,
    /// The amount of memory available to the function in MB.
    pub memory: i32,
    /// The version of the function being executed.
    pub version: String,
    /// The name of the Amazon CloudWatch Logs stream for the function.
    pub log_stream: String,
    /// The name of the Amazon CloudWatch Logs group for the function.
    pub log_group: String,
}

type RefConfig = Arc<Config>;

impl Config {
    /// Attempts to read configuration from environment variables.
    pub fn from_env() -> Self {
        Config {
            function_name: env::var("AWS_LAMBDA_FUNCTION_NAME").expect("Missing AWS_LAMBDA_FUNCTION_NAME env var"),
            memory: env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")
                .expect("Missing AWS_LAMBDA_FUNCTION_MEMORY_SIZE env var")
                .parse::<i32>()
                .expect("AWS_LAMBDA_FUNCTION_MEMORY_SIZE env var is not <i32>"),
            version: env::var("AWS_LAMBDA_FUNCTION_VERSION").expect("Missing AWS_LAMBDA_FUNCTION_VERSION env var"),
            log_stream: env::var("AWS_LAMBDA_LOG_STREAM_NAME").unwrap_or_default(),
            log_group: env::var("AWS_LAMBDA_LOG_GROUP_NAME").unwrap_or_default(),
        }
    }
}

/// Return a new [`ServiceFn`] with a closure that takes an event and context as separate arguments.
#[deprecated(since = "0.5.0", note = "Use `service_fn` and `LambdaEvent` instead")]
pub fn handler_fn<A, F, Fut>(f: F) -> ServiceFn<impl Fn(LambdaEvent<A>) -> Fut>
where
    F: Fn(A, Context) -> Fut,
{
    service_fn(move |req: LambdaEvent<A>| f(req.payload, req.context))
}

/// Starts the Lambda Rust runtime and begins polling for events on the [Lambda
/// Runtime APIs](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).
///
/// If you need more control over the runtime and add custom middleware, use the
/// [Runtime] type directly.
///
/// # Managed concurrency
/// If `AWS_LAMBDA_MAX_CONCURRENCY` is set, a warning is logged.
/// If your handler can satisfy `Clone + Send + 'static`,
/// prefer [`run_concurrent`] (requires the `concurrency-tokio` feature),
/// which honors managed concurrency and falls back to sequential behavior when
/// unset.
///
/// # Example
/// ```no_run
/// use lambda_runtime::{Error, service_fn, LambdaEvent};
/// use serde_json::Value;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let func = service_fn(func);
///     lambda_runtime::run(func).await?;
///     Ok(())
/// }
///
/// async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
///     Ok(event.payload)
/// }
/// ```
///
/// # Panics
///
/// This function panics if required Lambda environment variables are missing
/// (`AWS_LAMBDA_FUNCTION_NAME`, `AWS_LAMBDA_FUNCTION_MEMORY_SIZE`,
/// `AWS_LAMBDA_FUNCTION_VERSION`, `AWS_LAMBDA_RUNTIME_API`).
pub async fn run<A, F, R, B, S, D, E>(handler: F) -> Result<(), Error>
where
    F: Service<LambdaEvent<A>, Response = R>,
    F::Future: Future<Output = Result<R, F::Error>>,
    F::Error: Into<Diagnostic> + fmt::Debug,
    A: for<'de> Deserialize<'de>,
    R: IntoFunctionResponse<B, S>,
    B: Serialize,
    S: Stream<Item = Result<D, E>> + Unpin + Send + 'static,
    D: Into<bytes::Bytes> + Send,
    E: Into<Error> + Send + Debug,
{
    let runtime = Runtime::new(handler).layer(layers::TracingLayer::new());
    runtime.run().await
}

/// Starts the Lambda Rust runtime in a mode that is compatible with
/// Lambda Managed Instances (concurrent invocations).
///
/// Requires the `concurrency-tokio` feature.
///
/// When `AWS_LAMBDA_MAX_CONCURRENCY` is set to a value greater than 1, this
/// will spawn `AWS_LAMBDA_MAX_CONCURRENCY` tokio worker tasks, each running its own
/// `/next` polling loop. When the environment variable is unset or `<= 1`, it
/// falls back to the same sequential behavior as [`run`], so the same handler
/// can run on both classic Lambda and Lambda Managed Instances.
///
/// If you need more control over the runtime and add custom middleware, use the
/// [Runtime] type directly.
///
/// # Example
/// ```no_run
/// use lambda_runtime::{Error, service_fn, LambdaEvent};
/// use serde_json::Value;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let func = service_fn(func);
///     lambda_runtime::run_concurrent(func).await?;
///     Ok(())
/// }
///
/// async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
///     Ok(event.payload)
/// }
/// ```
///
/// # Panics
///
/// This function panics if:
/// - Called outside of a Tokio runtime
/// - Required Lambda environment variables are missing (`AWS_LAMBDA_FUNCTION_NAME`,
///   `AWS_LAMBDA_FUNCTION_MEMORY_SIZE`, `AWS_LAMBDA_FUNCTION_VERSION`,
///   `AWS_LAMBDA_RUNTIME_API`)
#[cfg(feature = "concurrency-tokio")]
#[cfg_attr(docsrs, doc(cfg(feature = "concurrency-tokio")))]
pub async fn run_concurrent<A, F, R, B, S, D, E>(handler: F) -> Result<(), Error>
where
    F: Service<LambdaEvent<A>, Response = R> + Clone + Send + 'static,
    F::Future: Future<Output = Result<R, F::Error>> + Send + 'static,
    F::Error: Into<Diagnostic> + fmt::Debug,
    A: for<'de> Deserialize<'de> + Send + 'static,
    R: IntoFunctionResponse<B, S> + Send + 'static,
    B: Serialize + Send + 'static,
    S: Stream<Item = Result<D, E>> + Unpin + Send + 'static,
    D: Into<bytes::Bytes> + Send + 'static,
    E: Into<Error> + Send + Debug + 'static,
{
    let runtime = Runtime::new(handler).layer(layers::TracingLayer::new());
    runtime.run_concurrent().await
}

/// Spawns a task that will be execute a provided async closure when the process
/// receives unix graceful shutdown signals. If the closure takes longer than 500ms
/// to execute, an unhandled `SIGKILL` signal might be received.
///
/// You can use this future to execute cleanup or flush related logic prior to runtime shutdown.
///
/// This function's returned future must be resolved prior to `lambda_runtime::run()`.
///
/// Note that this implicitly also registers and drives a no-op internal extension that subscribes to no events.
/// This extension will be named `_lambda-rust-runtime-no-op-graceful-shutdown-helper`. This extension name
/// can not be reused by other registered extensions. This is necessary in order to receive graceful shutdown signals.
///
/// This extension is cheap to run because it receives no events, but is not zero cost. If you have another extension
/// registered already, you might prefer to manually construct your own graceful shutdown handling without the dummy extension.
///
/// For more information on general AWS Lambda graceful shutdown handling, see:
/// <https://github.com/aws-samples/graceful-shutdown-with-aws-lambda>
///
/// # Panics
///
/// This function panics if:
/// - this function is called after `lambda_runtime::run()`
/// - this function is called outside of a context that has access to the tokio i/o
/// - the no-op extension cannot be registered
/// - either signal listener panics [tokio::signal::unix](https://docs.rs/tokio/latest/tokio/signal/unix/fn.signal.html#errors)
///
/// # Example
/// ```no_run
/// use lambda_runtime::{Error, service_fn, LambdaEvent};
/// use serde_json::Value;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Error> {
///     let func = service_fn(func);
///
///     let (writer, log_guard) = tracing_appender::non_blocking(std::io::stdout());
///     lambda_runtime::tracing::init_default_subscriber_with_writer(writer);
///
///     let shutdown_hook = || async move {
///         std::mem::drop(log_guard);
///     };
///     lambda_runtime::spawn_graceful_shutdown_handler(shutdown_hook).await;
///
///     lambda_runtime::run(func).await?;
///     Ok(())
/// }
///
/// async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
///     Ok(event.payload)
/// }
/// ```
#[cfg(all(unix, feature = "graceful-shutdown"))]
#[cfg_attr(docsrs, doc(cfg(all(unix, feature = "graceful-shutdown"))))]
pub async fn spawn_graceful_shutdown_handler<Fut>(shutdown_hook: impl FnOnce() -> Fut + Send + 'static)
where
    Fut: Future<Output = ()> + Send + 'static,
{
    // You need an extension registered with the Lambda orchestrator in order for your process
    // to receive a SIGTERM for graceful shutdown.
    //
    // We accomplish this here by registering a no-op internal extension, which does not subscribe to any events.
    //
    // This extension is cheap to run since after it connects to the lambda orchestration, the connection
    // will just wait forever for data to come, which never comes, so it won't cause wakes.
    let extension = lambda_extension::Extension::new()
        // Don't subscribe to any event types
        .with_events(&[])
        // Internal extension names MUST be unique within a given Lambda function.
        .with_extension_name("_lambda-rust-runtime-no-op-graceful-shutdown-helper")
        // Extensions MUST be registered before calling lambda_runtime::run(), which ends the Init
        // phase and begins the Invoke phase.
        .register()
        .await
        .expect("could not register no-op extension for graceful shutdown");

    tokio::task::spawn(async move {
        let graceful_shutdown_future = async move {
            let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();
            let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
            tokio::select! {
                _sigint = sigint.recv() => {
                    eprintln!("[runtime] SIGINT received");
                    eprintln!("[runtime] Graceful shutdown in progress ...");
                    shutdown_hook().await;
                    eprintln!("[runtime] Graceful shutdown completed");
                    std::process::exit(0);
                },
                _sigterm = sigterm.recv()=> {
                    eprintln!("[runtime] SIGTERM received");
                    eprintln!("[runtime] Graceful shutdown in progress ...");
                    shutdown_hook().await;
                    eprintln!("[runtime] Graceful shutdown completed");
                    std::process::exit(0);
                },
            }
        };

        let _: (_, ()) = tokio::join!(
            // we always poll the graceful shutdown future first,
            // which results in a smaller future due to lack of bookkeeping of which was last polled
            biased;
            graceful_shutdown_future, async {
            // we suppress extension errors because we don't actually mind if it crashes,
            // all we need to do is kick off the run so that lambda exits the init phase
            let _ = extension.run().await;
        });
    });
}
