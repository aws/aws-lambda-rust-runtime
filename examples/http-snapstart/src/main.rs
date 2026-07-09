// This example demonstrates using SnapStart with the lambda_http crate.
//
// Use lambda_http::runtime() to get a Runtime on which you can register
// `SnapStartResource`s. This gives you access to the SnapStart lifecycle while
// keeping the lambda_http request/response ergonomics.
//
// For the simple case where no custom hooks are needed, just use
// lambda_http::run() — SnapStart works automatically.

use lambda_http::{
    service_fn, tracing, BoxFuture, Body, Error, IntoResponse, Request, RequestExt, Response, SnapStartResource,
};
use std::sync::Arc;
use tokio::sync::RwLock;

struct DbPool {
    connected: bool,
}

impl DbPool {
    async fn connect() -> Self {
        tracing::info!("Establishing database connection pool");
        Self { connected: true }
    }
}

/// A `SnapStartResource` wrapper around the shared connection pool. The handler
/// and the resource share the same `Arc<RwLock<DbPool>>`, so draining before the
/// snapshot and reconnecting after restore are visible to invocations.
struct PoolResource(Arc<RwLock<DbPool>>);

impl SnapStartResource for PoolResource {
    fn before_snapshot(&self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            tracing::info!("Draining database connections before snapshot");
            self.0.write().await.connected = false;
            Ok(())
        })
    }

    fn after_restore(&self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            tracing::info!("Re-establishing database connections after restore");
            self.0.write().await.connected = true;
            Ok(())
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    let pool = Arc::new(RwLock::new(DbPool::connect().await));

    let pool_ref = pool.clone();
    let handler = service_fn(move |req: Request| {
        let pool = pool_ref.clone();
        async move {
            let pool = pool.read().await;
            assert!(pool.connected, "Pool should be connected during invocation");

            let name = req
                .query_string_parameters_ref()
                .and_then(|params| params.first("name"))
                .unwrap_or("world");

            Ok::<Response<Body>, Error>(format!("Hello, {name}!").into_response().await)
        }
    });

    lambda_http::runtime(handler)
        .register_snapstart_resource(Arc::new(PoolResource(pool.clone())))
        .run()
        .await?;
    Ok(())
}
