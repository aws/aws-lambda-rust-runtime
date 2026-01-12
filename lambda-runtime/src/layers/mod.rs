// Internally used services.
mod api_client;
mod api_response;
mod panic;

// Publicly available services.
/// Tracing utilities for Lambda runtime.
pub mod trace;

pub(crate) use api_client::RuntimeApiClientService;
pub(crate) use api_response::RuntimeApiResponseService;
pub(crate) use panic::CatchPanicService;
pub use trace::TracingLayer;

#[cfg(feature = "opentelemetry")]
mod otel;
#[cfg(feature = "opentelemetry")]
#[cfg_attr(docsrs, doc(cfg(feature = "opentelemetry")))]
pub use otel::{OpenTelemetryFaasTrigger, OpenTelemetryLayer};
