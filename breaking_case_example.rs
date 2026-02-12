// This is a customer's code that would break when RuntimeApiClientFuture
// changes from RuntimeApiClientFuture<F> to RuntimeApiClientFuture<F, B>

use lambda_runtime::{service_fn, Error, LambdaEvent, Runtime};
use serde_json::Value;
use tower::{Layer, Service};
use std::task::{Context, Poll};
use std::future::Future;
use std::pin::Pin;

// Customer creates a custom middleware layer
pub struct LoggingLayer;

impl<S> Layer<S> for LoggingLayer {
    type Service = LoggingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggingService { inner }
    }
}

pub struct LoggingService<S> {
    inner: S,
}

// Here's where the breaking change happens!
// The customer implements Service and explicitly names the Future type
impl<S> Service<lambda_runtime::LambdaInvocation> for LoggingService<S>
where
    S: Service<lambda_runtime::LambdaInvocation, Response = ()>,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Future<Output = Result<(), S::Error>> + Send + 'static,
{
    type Response = ();
    type Error = S::Error;
    
    // BREAKING CASE 1: Wrapping the future with explicit type annotation
    type Future = Pin<Box<dyn Future<Output = Result<(), S::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: lambda_runtime::LambdaInvocation) -> Self::Future {
        println!("Processing invocation: {}", req.context.request_id);
        
        let future = self.inner.call(req);
        
        // BREAKING CASE 2: If customer tries to inspect the future type
        // They might write code that depends on the concrete type
        Box::pin(async move {
            let result = future.await;
            println!("Invocation completed");
            result
        })
    }
}

// BREAKING CASE 3: Customer writes a helper function that explicitly
// constrains the Future type based on what they observed
pub fn create_runtime_with_logging<F, EventPayload, Response>(
    handler: F,
) -> Runtime<
    LoggingService<
        // This type signature explicitly names RuntimeApiClientService
        // and its associated types, which would break when generics change
        impl Service<
            lambda_runtime::LambdaInvocation,
            Response = (),
            Error = lambda_runtime_api_client::BoxError,
            // The Future type here implicitly depends on RuntimeApiClientFuture
        >
    >
>
where
    F: Service<LambdaEvent<EventPayload>, Response = Response>,
    F::Future: Future<Output = Result<Response, F::Error>>,
    F::Error: Into<lambda_runtime::Diagnostic> + std::fmt::Debug,
    EventPayload: for<'de> serde::Deserialize<'de>,
    Response: lambda_runtime::IntoFunctionResponse<Value, futures::stream::Empty<Result<bytes::Bytes, std::io::Error>>>,
{
    Runtime::new(handler).layer(LoggingLayer)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let handler = service_fn(my_handler);
    
    let runtime = create_runtime_with_logging(handler);
    
    runtime.run().await?;
    Ok(())
}

async fn my_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    Ok(event.payload)
}
