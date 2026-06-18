// This example demonstrates using SnapStart with the lambda_runtime crate.
//
// Implement the `SnapStartResource` trait on the types that hold
// snapshot-sensitive state (connections, credentials, cached values) and
// register them on the runtime. When deployed with SnapStart enabled, the
// runtime will:
// 1. Initialize and create resources (e.g., database connections)
// 2. Run each resource's `before_snapshot` hook (reverse registration order)
//    before the VM snapshot
// 3. Call `/restore/next`, which blocks until the VM is restored
// 4. Run each resource's `after_restore` hook (registration order) after restore
// 5. Enter the normal invocation loop
//
// When SnapStart is NOT enabled, the hooks are never called and the runtime
// behaves exactly as before.

use lambda_runtime::{service_fn, tracing, BoxFuture, Error, LambdaEvent, Runtime, SnapStartResource};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Deserialize)]
struct Request {
    name: String,
}

#[derive(Serialize)]
struct Response {
    message: String,
    invocation_count: u64,
}

struct AppState {
    counter: AtomicU64,
}

impl AppState {
    fn new() -> Self {
        Self {
            counter: AtomicU64::new(0),
        }
    }
}

impl SnapStartResource for AppState {
    fn before_snapshot(&self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            tracing::info!("Releasing resources before snapshot");
            self.counter.store(0, Ordering::Relaxed);
            Ok(())
        })
    }

    fn after_restore(&self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            tracing::info!("Re-establishing resources after restore");
            Ok(())
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let state = Arc::new(AppState::new());

    let state_ref = state.clone();
    let handler = service_fn(move |event: LambdaEvent<Request>| {
        let state = state_ref.clone();
        async move {
            let count = state.counter.fetch_add(1, Ordering::Relaxed) + 1;
            Ok::<_, Error>(Response {
                message: format!("Hello, {}!", event.payload.name),
                invocation_count: count,
            })
        }
    });

    Runtime::new(handler)
        .register_snapstart_resource(state.clone())
        .run()
        .await?;
    Ok(())
}
