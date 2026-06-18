#![deny(clippy::all, clippy::cargo)]
#![warn(missing_docs, nonstandard_style, rust_2018_idioms)]
#![allow(clippy::multiple_crate_versions)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! This crate includes a base HTTP client to interact with
//! the AWS Lambda Runtime API.
use futures_util::{future::BoxFuture, FutureExt, TryFutureExt};
use http::{
    uri::{PathAndQuery, Scheme},
    Request, Response, Uri,
};
use hyper::body::Incoming;
use hyper_util::client::legacy::connect::HttpConnector;
use std::{convert::TryInto, fmt::Debug, future, sync::OnceLock};

const USER_AGENT_HEADER: &str = "User-Agent";
const DEFAULT_USER_AGENT: &str = concat!("aws-lambda-rust/", env!("CARGO_PKG_VERSION"));
const CUSTOM_USER_AGENT: Option<&str> = option_env!("LAMBDA_RUNTIME_USER_AGENT");

mod error;
pub use error::*;
pub mod body;

#[cfg(feature = "tracing")]
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
pub mod tracing;

/// The single method the runtime needs from any client: send a request to the
/// Lambda Runtime API (RAPID). Implemented by both [`Client`] and [`PooledClient`].
pub trait RuntimeApiClient {
    /// Send a given request to the Runtime API.
    fn call(&self, req: Request<body::Body>) -> BoxFuture<'static, Result<Response<Incoming>, BoxError>>;
}

/// API client to interact with the AWS Lambda Runtime API.
///
/// **Superseded by [`PooledClient`]**, which additionally supports rebuilding its
/// connection pool after a SnapStart VM restore via [`PooledClient::reset_pool`].
/// `Client` remains fully functional; it simply cannot reset its pool after a
/// restore. Prefer [`PooledClient`] for new code; `Client` is expected to be
/// removed in the next major version (2.0.0).
#[derive(Debug)]
pub struct Client {
    /// The runtime API URI
    pub base: Uri,
    /// The client that manages the API connections
    pub client: hyper_util::client::legacy::Client<HttpConnector, body::Body>,
}

impl Client {
    /// Create a builder struct to configure the client.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Send a given request to the Runtime API.
    /// Use the client's base URI to ensure the API endpoint is correct.
    pub fn call(&self, req: Request<body::Body>) -> BoxFuture<'static, Result<Response<Incoming>, BoxError>> {
        <Self as RuntimeApiClient>::call(self, req)
    }

    /// Create a new client with a given base URI, HTTP connector, and optional pool size hint.
    fn with(base: Uri, connector: HttpConnector, pool_size: Option<usize>) -> Self {
        Self {
            base,
            client: build_hyper_client(&connector, pool_size),
        }
    }
}

impl RuntimeApiClient for Client {
    fn call(&self, req: Request<body::Body>) -> BoxFuture<'static, Result<Response<Incoming>, BoxError>> {
        // NOTE: This method returns a boxed future such that the future has a static lifetime.
        //       Due to limitations around the Rust async implementation as of Mar 2024, this is
        //       required to minimize constraints on the handler passed to [lambda_runtime::run].
        let req = match set_origin(&self.base, req) {
            Ok(req) => req,
            Err(err) => return future::ready(Err(err)).boxed(),
        };
        self.client.request(req).map_err(Into::into).boxed()
    }
}

/// API client to interact with the AWS Lambda Runtime API, with support for
/// rebuilding its connection pool after a SnapStart VM restore.
#[derive(Debug)]
pub struct PooledClient {
    /// The runtime API URI
    pub base: Uri,
    /// The client that manages the API connections
    client: hyper_util::client::legacy::Client<HttpConnector, body::Body>,
    /// The HTTP connector used to rebuild the client after a SnapStart restore
    connector: HttpConnector,
    /// Optional pool size hint for the hyper client
    pool_size: Option<usize>,
    /// Holds a freshly built client after a SnapStart restore. When set, [`PooledClient::call`]
    /// uses it instead of [`PooledClient::client`]; otherwise the original client is used.
    ///
    /// This is populated exactly once by [`PooledClient::reset_pool`] during the restore
    /// lifecycle, before any concurrent polling starts, so the `OnceLock` write never
    /// races with reads. Using `OnceLock` keeps `call` lock-free on the hot path.
    restored: OnceLock<hyper_util::client::legacy::Client<HttpConnector, body::Body>>,
}

impl PooledClient {
    /// Create a builder struct to configure the client.
    pub fn builder() -> PooledClientBuilder {
        PooledClientBuilder::new()
    }

    /// Send a given request to the Runtime API.
    /// Use the client's base URI to ensure the API endpoint is correct.
    pub fn call(&self, req: Request<body::Body>) -> BoxFuture<'static, Result<Response<Incoming>, BoxError>> {
        <Self as RuntimeApiClient>::call(self, req)
    }

    /// Create a new client with a given base URI, HTTP connector, and optional pool size hint.
    fn with(base: Uri, connector: HttpConnector, pool_size: Option<usize>) -> Self {
        let client = build_hyper_client(&connector, pool_size);
        Self {
            base,
            client,
            connector,
            pool_size,
            restored: OnceLock::new(),
        }
    }

    /// Reset the internal connection pool by building a fresh hyper client that
    /// subsequent calls will use.
    ///
    /// This is useful after a SnapStart VM restore where existing connections
    /// to the Lambda Runtime API (RAPID) are stale and must be discarded. It is
    /// called exactly once during the restore lifecycle, before concurrent
    /// polling starts, so the underlying `OnceLock` write never races.
    pub fn reset_pool(&self) {
        let _ = self.restored.set(build_hyper_client(&self.connector, self.pool_size));
    }
}

impl RuntimeApiClient for PooledClient {
    fn call(&self, req: Request<body::Body>) -> BoxFuture<'static, Result<Response<Incoming>, BoxError>> {
        let req = match set_origin(&self.base, req) {
            Ok(req) => req,
            Err(err) => return future::ready(Err(err)).boxed(),
        };
        // After a SnapStart restore, `restored` holds a fresh client with no stale
        // connections; otherwise fall back to the original client. Lock-free read.
        let hyper_client = self.restored.get().unwrap_or(&self.client);
        hyper_client.request(req).map_err(Into::into).boxed()
    }
}

/// Build a fresh hyper client from the given connector and pool size settings.
fn build_hyper_client(
    connector: &HttpConnector,
    pool_size: Option<usize>,
) -> hyper_util::client::legacy::Client<HttpConnector, body::Body> {
    let mut builder = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new());
    builder.http1_max_buf_size(1024 * 1024);

    if let Some(size) = pool_size {
        builder.pool_max_idle_per_host(size);
    }

    builder.build(connector.clone())
}

/// Rewrite a request's URI against the client's base (scheme/authority/base path).
fn set_origin<B>(base: &Uri, req: Request<B>) -> Result<Request<B>, BoxError> {
    let (mut parts, body) = req.into_parts();
    let (scheme, authority, base_path) = {
        let scheme = base.scheme().unwrap_or(&Scheme::HTTP);
        let authority = base.authority().expect("Authority not found");
        let base_path = base.path().trim_end_matches('/');
        (scheme, authority, base_path)
    };
    let path = parts.uri.path_and_query().expect("PathAndQuery not found");
    let pq: PathAndQuery = format!("{base_path}{path}").parse().expect("PathAndQuery invalid");

    let uri = Uri::builder()
        .scheme(scheme.as_ref())
        .authority(authority.as_ref())
        .path_and_query(pq)
        .build()
        .map_err(Box::new)?;

    parts.uri = uri;
    Ok(Request::from_parts(parts, body))
}

/// Builder implementation to construct any Runtime API clients.
pub struct ClientBuilder {
    connector: HttpConnector,
    uri: Option<http::Uri>,
    pool_size: Option<usize>,
}

impl ClientBuilder {
    fn new() -> ClientBuilder {
        ClientBuilder {
            connector: HttpConnector::new(),
            uri: None,
            pool_size: None,
        }
    }

    /// Create a new builder with a given HTTP connector.
    pub fn with_connector(self, connector: HttpConnector) -> ClientBuilder {
        ClientBuilder {
            connector,
            uri: self.uri,
            pool_size: self.pool_size,
        }
    }

    /// Create a new builder with a given base URI.
    /// Inherits all other attributes from the existent builder.
    pub fn with_endpoint(self, uri: http::Uri) -> Self {
        Self { uri: Some(uri), ..self }
    }

    /// Provide a pool size hint for the underlying Hyper client.
    ///
    /// When using concurrent polling, this should be at least the maximum
    /// concurrency (e.g., `AWS_LAMBDA_MAX_CONCURRENCY`) to avoid connection
    /// starvation.
    pub fn with_pool_size(self, pool_size: usize) -> Self {
        Self {
            pool_size: Some(pool_size),
            ..self
        }
    }

    /// Create the new client to interact with the Runtime API.
    ///
    /// **Superseded by [`PooledClient::builder`]**, which produces a
    /// [`PooledClient`] that supports SnapStart connection-pool reset. Prefer
    /// `PooledClient::builder().build()` for new code; this method is expected to
    /// be removed in the next major version (2.0.0).
    pub fn build(self) -> Result<Client, Error> {
        let uri = resolve_uri(self.uri);
        Ok(Client::with(uri, self.connector, self.pool_size))
    }
}

/// Builder to construct a [`PooledClient`].
pub struct PooledClientBuilder {
    connector: HttpConnector,
    uri: Option<http::Uri>,
    pool_size: Option<usize>,
}

impl PooledClientBuilder {
    fn new() -> PooledClientBuilder {
        PooledClientBuilder {
            connector: HttpConnector::new(),
            uri: None,
            pool_size: None,
        }
    }

    /// Use a given HTTP connector.
    pub fn with_connector(self, connector: HttpConnector) -> PooledClientBuilder {
        PooledClientBuilder { connector, ..self }
    }

    /// Use a given base URI. Inherits all other attributes from the existing builder.
    pub fn with_endpoint(self, uri: http::Uri) -> Self {
        Self { uri: Some(uri), ..self }
    }

    /// Provide a pool size hint for the underlying Hyper client.
    ///
    /// When using concurrent polling, this should be at least the maximum
    /// concurrency (e.g., `AWS_LAMBDA_MAX_CONCURRENCY`) to avoid connection
    /// starvation.
    pub fn with_pool_size(self, pool_size: usize) -> Self {
        Self {
            pool_size: Some(pool_size),
            ..self
        }
    }

    /// Create the new [`PooledClient`] to interact with the Runtime API.
    pub fn build(self) -> PooledClient {
        let uri = resolve_uri(self.uri);
        PooledClient::with(uri, self.connector, self.pool_size)
    }
}

/// Resolve the base URI from an explicit value or the `AWS_LAMBDA_RUNTIME_API` env var.
fn resolve_uri(uri: Option<http::Uri>) -> Uri {
    match uri {
        Some(uri) => uri,
        None => {
            let uri = std::env::var("AWS_LAMBDA_RUNTIME_API").expect("Missing AWS_LAMBDA_RUNTIME_API env var");
            uri.try_into().expect("Unable to convert to URL")
        }
    }
}

/// Create a request builder.
/// This builder uses `aws-lambda-rust/CRATE_VERSION` as
/// the default User-Agent.
/// Configure environment variable `LAMBDA_RUNTIME_USER_AGENT`
/// at compile time to modify User-Agent value.
pub fn build_request() -> http::request::Builder {
    const USER_AGENT: &str = match CUSTOM_USER_AGENT {
        Some(value) => value,
        None => DEFAULT_USER_AGENT,
    };
    http::Request::builder().header(USER_AGENT_HEADER, USER_AGENT)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_origin() {
        let base: Uri = "http://localhost:9001".parse().unwrap();
        let req = build_request()
            .uri("/2018-06-01/runtime/invocation/next")
            .body(())
            .unwrap();
        let req = set_origin(&base, req).unwrap();
        assert_eq!(
            "http://localhost:9001/2018-06-01/runtime/invocation/next",
            &req.uri().to_string()
        );
    }

    #[test]
    fn test_set_origin_with_base_path() {
        let base: Uri = "http://localhost:9001/foo".parse().unwrap();
        let req = build_request()
            .uri("/2018-06-01/runtime/invocation/next")
            .body(())
            .unwrap();
        let req = set_origin(&base, req).unwrap();
        assert_eq!(
            "http://localhost:9001/foo/2018-06-01/runtime/invocation/next",
            &req.uri().to_string()
        );

        let base: Uri = "http://localhost:9001/foo/".parse().unwrap();
        let req = build_request()
            .uri("/2018-06-01/runtime/invocation/next")
            .body(())
            .unwrap();
        let req = set_origin(&base, req).unwrap();
        assert_eq!(
            "http://localhost:9001/foo/2018-06-01/runtime/invocation/next",
            &req.uri().to_string()
        );
    }

    #[test]
    fn builder_accepts_pool_size() {
        let base = "http://localhost:9001";
        let expected: Uri = base.parse().unwrap();
        let client = PooledClient::builder()
            .with_pool_size(4)
            .with_endpoint(base.parse().unwrap())
            .build();

        assert_eq!(client.base, expected);
    }

    #[test]
    fn test_reset_pool() {
        let base = "http://localhost:9001";
        let client = PooledClient::builder()
            .with_pool_size(4)
            .with_endpoint(base.parse().unwrap())
            .build();
        client.reset_pool();
        assert_eq!(client.base, base.parse::<Uri>().unwrap());
    }
}
