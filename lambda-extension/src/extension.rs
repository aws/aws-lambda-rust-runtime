use http::Request;
use http_body_util::BodyExt;
use hyper::{body::Incoming, server::conn::http1, service::service_fn};

use hyper_util::rt::tokio::TokioIo;
use lambda_runtime_api_client::Client;
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    convert::Infallible,
    fmt,
    future::{ready, Future},
    marker::PhantomData,
    net::SocketAddr,
    path::PathBuf,
    pin::Pin,
    sync::Arc,
};
use tokio::{net::TcpListener, sync::Mutex};
use tokio_stream::StreamExt;
use tower::{MakeService, Service, ServiceExt};
use tracing::trace;

use crate::{
    logs::*,
    requests::{self, Api},
    telemetry_wrapper, Error, ExtensionError, LambdaEvent, LambdaTelemetry, NextEvent,
};

const DEFAULT_LOG_PORT_NUMBER: u16 = 9002;
const DEFAULT_TELEMETRY_PORT_NUMBER: u16 = 9003;

/// An Extension that runs event, log and telemetry processors
pub struct Extension<'a, E, L, T, TL = String> {
    extension_name: Option<&'a str>,
    events: Option<&'a [&'a str]>,
    events_processor: E,
    log_types: Option<&'a [&'a str]>,
    logs_processor: Option<L>,
    log_buffering: Option<LogBuffering>,
    log_port_number: u16,
    telemetry_types: Option<&'a [&'a str]>,
    telemetry_processor: Option<T>,
    telemetry_buffering: Option<LogBuffering>,
    telemetry_port_number: u16,
    _telemetry_record_type: PhantomData<fn(TL)>,
}

impl Extension<'_, Identity<LambdaEvent>, MakeIdentity<Vec<LambdaLog>>, MakeIdentity<Vec<LambdaTelemetry>>> {
    /// Create a new base [`Extension`] with a no-op events processor
    pub fn new() -> Self {
        Extension {
            extension_name: None,
            events: None,
            events_processor: Identity::new(),
            log_types: None,
            log_buffering: None,
            logs_processor: None,
            log_port_number: DEFAULT_LOG_PORT_NUMBER,
            telemetry_types: None,
            telemetry_buffering: None,
            telemetry_processor: None,
            telemetry_port_number: DEFAULT_TELEMETRY_PORT_NUMBER,
            _telemetry_record_type: PhantomData,
        }
    }
}

impl Default
    for Extension<'_, Identity<LambdaEvent>, MakeIdentity<Vec<LambdaLog>>, MakeIdentity<Vec<LambdaTelemetry>>>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E, L, T, TL> Extension<'a, E, L, T, TL>
where
    E: Service<LambdaEvent>,
    E::Future: Future<Output = Result<(), E::Error>>,
    E::Error: Into<Error> + fmt::Display + fmt::Debug,

    // Fixme: 'static bound might be too restrictive
    L: MakeService<(), Vec<LambdaLog>, Response = ()> + Send + Sync + 'static,
    L::Service: Service<Vec<LambdaLog>, Response = ()> + Send + Sync,
    <L::Service as Service<Vec<LambdaLog>>>::Future: Send + 'a,
    L::Error: Into<Error> + fmt::Debug,
    L::MakeError: Into<Error> + fmt::Debug,
    L::Future: Send,

    // Fixme: 'static bound might be too restrictive
    T: MakeService<(), Vec<LambdaTelemetry<TL>>, Response = ()> + Send + Sync + 'static,
    T::Service: Service<Vec<LambdaTelemetry<TL>>, Response = ()> + Send + Sync,
    <T::Service as Service<Vec<LambdaTelemetry<TL>>>>::Future: Send + 'a,
    T::Error: Into<Error> + fmt::Debug,
    T::MakeError: Into<Error> + fmt::Debug,
    T::Future: Send,
    TL: DeserializeOwned + Send + 'static,
{
    /// Create a new [`Extension`] with a given extension name
    pub fn with_extension_name(self, extension_name: &'a str) -> Self {
        Extension {
            extension_name: Some(extension_name),
            ..self
        }
    }

    /// Create a new [`Extension`] with a list of given events.
    /// The only accepted events are `INVOKE` and `SHUTDOWN`.
    pub fn with_events(self, events: &'a [&'a str]) -> Self {
        Extension {
            events: Some(events),
            ..self
        }
    }

    /// Create a new [`Extension`] with a service that receives Lambda events.
    pub fn with_events_processor<N>(self, ep: N) -> Extension<'a, N, L, T, TL>
    where
        N: Service<LambdaEvent>,
        N::Future: Future<Output = Result<(), N::Error>>,
        N::Error: Into<Error> + fmt::Display,
    {
        Extension {
            events_processor: ep,
            extension_name: self.extension_name,
            events: self.events,
            log_types: self.log_types,
            log_buffering: self.log_buffering,
            logs_processor: self.logs_processor,
            log_port_number: self.log_port_number,
            telemetry_types: self.telemetry_types,
            telemetry_buffering: self.telemetry_buffering,
            telemetry_processor: self.telemetry_processor,
            telemetry_port_number: self.telemetry_port_number,
            _telemetry_record_type: self._telemetry_record_type,
        }
    }

    /// Create a new [`Extension`] with a service that receives Lambda logs.
    pub fn with_logs_processor<N, NS>(self, lp: N) -> Extension<'a, E, N, T, TL>
    where
        N: Service<()>,
        N::Future: Future<Output = Result<NS, N::Error>>,
        N::Error: Into<Error> + fmt::Display,
    {
        Extension {
            logs_processor: Some(lp),
            events_processor: self.events_processor,
            extension_name: self.extension_name,
            events: self.events,
            log_types: self.log_types,
            log_buffering: self.log_buffering,
            log_port_number: self.log_port_number,
            telemetry_types: self.telemetry_types,
            telemetry_buffering: self.telemetry_buffering,
            telemetry_processor: self.telemetry_processor,
            telemetry_port_number: self.telemetry_port_number,
            _telemetry_record_type: self._telemetry_record_type,
        }
    }

    /// Create a new [`Extension`] with a list of logs types to subscribe.
    /// The only accepted log types are `function`, `platform`, and `extension`.
    pub fn with_log_types(self, log_types: &'a [&'a str]) -> Self {
        Extension {
            log_types: Some(log_types),
            ..self
        }
    }

    /// Create a new [`Extension`] with specific configuration to buffer logs.
    pub fn with_log_buffering(self, lb: LogBuffering) -> Self {
        Extension {
            log_buffering: Some(lb),
            ..self
        }
    }

    /// Create a new [`Extension`] with a different port number to listen to logs.
    pub fn with_log_port_number(self, port_number: u16) -> Self {
        Extension {
            log_port_number: port_number,
            ..self
        }
    }

    /// Create a new [`Extension`] with a service that receives Lambda telemetry data.
    ///
    /// By default, telemetry log records are deserialized as `String`, but
    /// it's possible to configure Lambda functions to emit logs in JSON format.
    /// For more information, refer to [`Self::with_telemetry_record_type`].
    pub fn with_telemetry_processor<N, NS>(self, lp: N) -> Extension<'a, E, L, N, TL>
    where
        N: Service<()>,
        N::Future: Future<Output = Result<NS, N::Error>>,
        N::Error: Into<Error> + fmt::Display,
    {
        Extension {
            telemetry_processor: Some(lp),
            events_processor: self.events_processor,
            extension_name: self.extension_name,
            events: self.events,
            log_types: self.log_types,
            log_buffering: self.log_buffering,
            logs_processor: self.logs_processor,
            log_port_number: self.log_port_number,
            telemetry_types: self.telemetry_types,
            telemetry_buffering: self.telemetry_buffering,
            telemetry_port_number: self.telemetry_port_number,
            _telemetry_record_type: self._telemetry_record_type,
        }
    }

    /// Create a new [`Extension`] with a list of telemetry types to subscribe.
    /// The only accepted telemetry types are `function`, `platform`, and `extension`.
    pub fn with_telemetry_types(self, telemetry_types: &'a [&'a str]) -> Self {
        Extension {
            telemetry_types: Some(telemetry_types),
            ..self
        }
    }

    /// Create a new [`Extension`] with specific configuration to buffer telemetry.
    pub fn with_telemetry_buffering(self, lb: LogBuffering) -> Self {
        Extension {
            telemetry_buffering: Some(lb),
            ..self
        }
    }

    /// Create a new [`Extension`] with a different port number to listen to telemetry.
    pub fn with_telemetry_port_number(self, port_number: u16) -> Self {
        Extension {
            telemetry_port_number: port_number,
            ..self
        }
    }

    /// Register the extension.
    ///
    /// Performs the
    /// [init phase](https://docs.aws.amazon.com/lambda/latest/dg/lambda-runtime-environment.html#runtimes-lifecycle-ib)
    /// Lambda lifecycle operations to register the extension. When implementing an internal Lambda
    /// extension, it is safe to call `lambda_runtime::run` once the future returned by this
    /// function resolves.
    pub async fn register(self) -> Result<RegisteredExtension<E>, Error> {
        let client = &Client::builder().build()?;

        let register_res = register(client, self.extension_name, self.events).await?;

        // Logs API subscriptions must be requested during the Lambda init phase (see
        // https://docs.aws.amazon.com/lambda/latest/dg/runtimes-logs-api.html#runtimes-logs-api-subscribing).
        if let Some(mut log_processor) = self.logs_processor {
            trace!("Log processor found");

            validate_buffering_configuration(self.log_buffering)?;

            let addr = SocketAddr::from(([0, 0, 0, 0], self.log_port_number));
            let service = log_processor.make_service(());
            let service = Arc::new(Mutex::new(service.await.unwrap()));
            tokio::task::spawn(async move {
                trace!("Creating new logs processor Service");

                loop {
                    let service: Arc<Mutex<_>> = service.clone();
                    let make_service = service_fn(move |req: Request<Incoming>| log_wrapper(service.clone(), req));

                    let listener = TcpListener::bind(addr).await.unwrap();
                    let (tcp, _) = listener.accept().await.unwrap();
                    let io = TokioIo::new(tcp);
                    tokio::task::spawn(async move {
                        if let Err(err) = http1::Builder::new().serve_connection(io, make_service).await {
                            println!("Error serving connection: {err:?}");
                        }
                    });
                }
            });

            trace!("Log processor started");

            // Call Logs API to start receiving events
            let req = requests::subscribe_request(
                Api::LogsApi,
                &register_res.extension_id,
                self.log_types,
                self.log_buffering,
                self.log_port_number,
            )?;
            let res = client.call(req).await?;
            if !res.status().is_success() {
                let err = format!("unable to initialize the logs api: {}", res.status());
                return Err(ExtensionError::boxed(err));
            }
            trace!("Registered extension with Logs API");
        }

        // Telemetry API subscriptions must be requested during the Lambda init phase (see
        // https://docs.aws.amazon.com/lambda/latest/dg/telemetry-api.html#telemetry-api-registration
        if let Some(mut telemetry_processor) = self.telemetry_processor {
            trace!("Telemetry processor found");

            validate_buffering_configuration(self.telemetry_buffering)?;

            let addr = SocketAddr::from(([0, 0, 0, 0], self.telemetry_port_number));
            let service = telemetry_processor.make_service(());
            let service = Arc::new(Mutex::new(service.await.unwrap()));
            tokio::task::spawn(async move {
                trace!("Creating new telemetry processor Service");

                loop {
                    let service = service.clone();
                    let make_service = service_fn(move |req| telemetry_wrapper(service.clone(), req));

                    let listener = TcpListener::bind(addr).await.unwrap();
                    let (tcp, _) = listener.accept().await.unwrap();
                    let io = TokioIo::new(tcp);
                    tokio::task::spawn(async move {
                        if let Err(err) = http1::Builder::new().serve_connection(io, make_service).await {
                            println!("Error serving connection: {err:?}");
                        }
                    });
                }
            });

            trace!("Telemetry processor started");

            // Call Telemetry API to start receiving events
            let req = requests::subscribe_request(
                Api::TelemetryApi,
                &register_res.extension_id,
                self.telemetry_types,
                self.telemetry_buffering,
                self.telemetry_port_number,
            )?;
            let res = client.call(req).await?;
            if !res.status().is_success() {
                let err = format!("unable to initialize the telemetry api: {}", res.status());
                return Err(ExtensionError::boxed(err));
            }
            trace!("Registered extension with Telemetry API");
        }

        Ok(RegisteredExtension {
            extension_id: register_res.extension_id,
            function_name: register_res.function_name,
            function_version: register_res.function_version,
            handler: register_res.handler,
            account_id: register_res.account_id,
            events_processor: self.events_processor,
        })
    }

    /// Execute the given extension.
    pub async fn run(self) -> Result<(), Error> {
        self.register().await?.run().await
    }
}

impl<'a, E, L> Extension<'a, E, L, MakeIdentity<Vec<LambdaTelemetry>>> {
    /// Set the deserialization type for telemetry log records.
    ///
    /// By default, telemetry log records are deserialized as `String`, but
    /// it's possible to configure Lambda functions to emit logs in JSON format.
    /// Use this method to deserialize into a different type, such as
    /// `serde_json::Value`.
    ///
    /// Must be called before [`Self::with_telemetry_processor`].
    ///
    /// ```
    /// use lambda_extension::{Extension, LambdaTelemetry, SharedService, service_fn};
    ///
    /// async fn handler(events: Vec<LambdaTelemetry<serde_json::Value>>) -> Result<(), lambda_extension::Error> {
    ///     for event in &events {
    ///         println!("{event:?}");
    ///     }
    ///     Ok(())
    /// }
    ///
    /// let _ext = Extension::new()
    ///     .with_telemetry_record_type::<serde_json::Value>()
    ///     .with_telemetry_processor(SharedService::new(service_fn(handler)));
    /// ```
    pub fn with_telemetry_record_type<N>(self) -> Extension<'a, E, L, MakeIdentity<Vec<LambdaTelemetry<N>>>, N> {
        Extension {
            _telemetry_record_type: PhantomData,
            telemetry_processor: None,
            events_processor: self.events_processor,
            extension_name: self.extension_name,
            events: self.events,
            log_types: self.log_types,
            log_buffering: self.log_buffering,
            logs_processor: self.logs_processor,
            log_port_number: self.log_port_number,
            telemetry_types: self.telemetry_types,
            telemetry_buffering: self.telemetry_buffering,
            telemetry_port_number: self.telemetry_port_number,
        }
    }
}

/// An extension registered by calling [`Extension::register`].
pub struct RegisteredExtension<E> {
    /// The ID of the registered extension. This ID is unique per extension and remains constant
    pub extension_id: String,
    /// The ID of the account the extension was registered to.
    /// This will be `None` if the register request doesn't send the Lambda-Extension-Accept-Feature header
    pub account_id: Option<String>,
    /// The name of the Lambda function that the extension is registered with
    pub function_name: String,
    /// The version of the Lambda function that the extension is registered with
    pub function_version: String,
    /// The Lambda function handler that AWS Lambda invokes
    pub handler: String,
    events_processor: E,
}

impl<E> RegisteredExtension<E>
where
    E: Service<LambdaEvent>,
    E::Future: Future<Output = Result<(), E::Error>>,
    E::Error: Into<Box<dyn std::error::Error + Send + Sync>> + fmt::Display + fmt::Debug,
{
    /// Execute the extension's run loop.
    ///
    /// Performs the
    /// [invoke](https://docs.aws.amazon.com/lambda/latest/dg/lambda-runtime-environment.html#runtimes-lifecycle-invoke)
    /// and, for external Lambda extensions registered to receive the `SHUTDOWN` event, the
    /// [shutdown](https://docs.aws.amazon.com/lambda/latest/dg/lambda-runtime-environment.html#runtimes-lifecycle-shutdown)
    /// Lambda lifecycle phases.
    pub async fn run(self) -> Result<(), Error> {
        let client = &Client::builder().build()?;
        let mut ep = self.events_processor;
        let extension_id = &self.extension_id;

        let incoming = async_stream::stream! {
            loop {
                trace!("Waiting for next event (incoming loop)");
                let req = requests::next_event_request(extension_id)?;
                let res = client.call(req).await;
                yield res;
            }
        };

        tokio::pin!(incoming);
        while let Some(event) = incoming.next().await {
            trace!("New event arrived (run loop)");
            let event = event?;
            let (_parts, body) = event.into_parts();

            let body = body.collect().await?.to_bytes();
            trace!("{}", std::str::from_utf8(&body)?); // this may be very verbose
            let event: NextEvent = serde_json::from_slice(&body)?;
            let is_invoke = event.is_invoke();

            let event = LambdaEvent::new(event);

            let ep = match ep.ready().await {
                Ok(ep) => ep,
                Err(err) => {
                    println!("Inner service is not ready: {err:?}");
                    let req = if is_invoke {
                        requests::init_error(extension_id, &err.to_string(), None)?
                    } else {
                        requests::exit_error(extension_id, &err.to_string(), None)?
                    };

                    client.call(req).await?;
                    return Err(err.into());
                }
            };

            let res = ep.call(event).await;
            if let Err(err) = res {
                println!("{err:?}");
                let req = if is_invoke {
                    requests::init_error(extension_id, &err.to_string(), None)?
                } else {
                    requests::exit_error(extension_id, &err.to_string(), None)?
                };

                client.call(req).await?;
                return Err(err.into());
            }
        }

        // Unreachable.
        Ok(())
    }
}

/// A no-op generic processor
#[derive(Clone)]
pub struct Identity<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Identity<T> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Service<T> for Identity<T> {
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
    type Response = ();

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _event: T) -> Self::Future {
        Box::pin(ready(Ok(())))
    }
}

/// Service factory to generate no-op generic processors
#[derive(Clone)]
pub struct MakeIdentity<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Service<()> for MakeIdentity<T>
where
    T: Send + Sync + 'static,
{
    type Error = Infallible;
    type Response = Identity<T>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut core::task::Context<'_>) -> core::task::Poll<Result<(), Self::Error>> {
        core::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: ()) -> Self::Future {
        Box::pin(ready(Ok(Identity::new())))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterResponseBody {
    function_name: String,
    function_version: String,
    handler: String,
    account_id: Option<String>,
}

#[derive(Debug)]
struct RegisterResponse {
    extension_id: String,
    function_name: String,
    function_version: String,
    handler: String,
    account_id: Option<String>,
}

/// Initialize and register the extension in the Extensions API
async fn register<'a>(
    client: &'a Client,
    extension_name: Option<&'a str>,
    events: Option<&'a [&'a str]>,
) -> Result<RegisterResponse, Error> {
    let name = match extension_name {
        Some(name) => name.into(),
        None => {
            let args: Vec<String> = std::env::args().collect();
            PathBuf::from(args[0].clone())
                .file_name()
                .expect("unexpected executable name")
                .to_str()
                .expect("unexpect executable name")
                .to_string()
        }
    };

    let events = events.unwrap_or(&["INVOKE", "SHUTDOWN"]);

    let req = requests::register_request(&name, events)?;
    let res = client.call(req).await?;
    if !res.status().is_success() {
        let err = format!("unable to register the extension: {}", res.status());
        return Err(ExtensionError::boxed(err));
    }

    let header = res
        .headers()
        .get(requests::EXTENSION_ID_HEADER)
        .ok_or_else(|| ExtensionError::boxed("missing extension id header"))
        .map_err(|e| ExtensionError::boxed(e.to_string()))?;
    let extension_id = header.to_str()?.to_string();

    let (_, body) = res.into_parts();
    let body = body.collect().await?.to_bytes();
    let response: RegisterResponseBody = serde_json::from_slice(&body)?;

    Ok(RegisterResponse {
        extension_id,
        function_name: response.function_name,
        function_version: response.function_version,
        handler: response.handler,
        account_id: response.account_id,
    })
}
