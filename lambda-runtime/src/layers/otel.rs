use std::{fmt::Display, future::Future, pin::Pin, task};

use crate::LambdaInvocation;
use opentelemetry_semantic_conventions::attribute;
use pin_project::pin_project;
use tower::{Layer, Service};
use tracing::{field, instrument::Instrumented, Instrument};

/// Tower layer to add OpenTelemetry tracing to a Lambda function invocation. The layer accepts
/// a function to flush OpenTelemetry after the end of the invocation.
pub struct OpenTelemetryLayer<F> {
    flush_fn: F,
    otel_attribute_trigger: OpenTelemetryFaasTrigger,
}

impl<F> OpenTelemetryLayer<F>
where
    F: Fn() + Clone,
{
    /// Create a new [OpenTelemetryLayer] with the provided flush function.
    pub fn new(flush_fn: F) -> Self {
        Self {
            flush_fn,
            otel_attribute_trigger: Default::default(),
        }
    }

    /// Configure the `faas.trigger` attribute of the OpenTelemetry span.
    pub fn with_trigger(self, trigger: OpenTelemetryFaasTrigger) -> Self {
        Self {
            otel_attribute_trigger: trigger,
            ..self
        }
    }
}

impl<S, F> Layer<S> for OpenTelemetryLayer<F>
where
    F: Fn() + Clone,
{
    type Service = OpenTelemetryService<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        OpenTelemetryService {
            inner,
            flush_fn: self.flush_fn.clone(),
            coldstart: true,
            otel_attribute_trigger: self.otel_attribute_trigger.to_string(),
        }
    }
}

/// Tower service created by [OpenTelemetryLayer].
pub struct OpenTelemetryService<S, F> {
    inner: S,
    flush_fn: F,
    coldstart: bool,
    otel_attribute_trigger: String,
}

impl<S, F> Service<LambdaInvocation> for OpenTelemetryService<S, F>
where
    S: Service<LambdaInvocation, Response = ()>,
    F: Fn() + Clone,
{
    type Error = S::Error;
    type Response = ();
    type Future = OpenTelemetryFuture<Instrumented<S::Future>, F>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        let span = if let Some(tenant_id) = &req.context.tenant_id {
            tracing::info_span!(
                "Lambda function invocation",
                "otel.name" = req.context.env_config.function_name,
                "otel.kind" = field::Empty,
                { attribute::FAAS_TRIGGER } = &self.otel_attribute_trigger,
                { attribute::FAAS_INVOCATION_ID } = req.context.request_id,
                { attribute::FAAS_COLDSTART } = self.coldstart,
                "tenant_id" = tenant_id
            )
        } else {
            tracing::info_span!(
                "Lambda function invocation",
                "otel.name" = req.context.env_config.function_name,
                "otel.kind" = field::Empty,
                { attribute::FAAS_TRIGGER } = &self.otel_attribute_trigger,
                { attribute::FAAS_INVOCATION_ID } = req.context.request_id,
                { attribute::FAAS_COLDSTART } = self.coldstart
            )
        };

        // After the first execution, we can set 'coldstart' to false
        self.coldstart = false;

        let future = {
            // Enter the span before calling the inner service
            // to ensure that it's assigned as parent of the inner spans.
            let _guard = span.enter();
            self.inner.call(req)
        };
        OpenTelemetryFuture {
            future: Some(future.instrument(span)),
            flush_fn: self.flush_fn.clone(),
        }
    }
}

/// Future created by [OpenTelemetryService].
#[pin_project]
pub struct OpenTelemetryFuture<Fut, F> {
    #[pin]
    future: Option<Fut>,
    flush_fn: F,
}

impl<Fut, F> Future for OpenTelemetryFuture<Fut, F>
where
    Fut: Future,
    F: Fn(),
{
    type Output = Fut::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        // First, try to get the ready value of the future
        let ready = task::ready!(self
            .as_mut()
            .project()
            .future
            .as_pin_mut()
            .expect("future polled after completion")
            .poll(cx));

        // If we got the ready value, we first drop the future: this ensures that the
        // OpenTelemetry span attached to it is closed and included in the subsequent flush.
        Pin::set(&mut self.as_mut().project().future, None);
        (self.project().flush_fn)();
        task::Poll::Ready(ready)
    }
}

/// Represent the possible values for the OpenTelemetry `faas.trigger` attribute.
/// See <https://opentelemetry.io/docs/specs/semconv/attributes-registry/faas/> for more details.
#[derive(Default, Clone, Copy)]
#[non_exhaustive]
pub enum OpenTelemetryFaasTrigger {
    /// A response to some data source operation such as a database or filesystem read/write
    #[default]
    Datasource,
    /// To provide an answer to an inbound HTTP request
    Http,
    /// A function is set to be executed when messages are sent to a messaging system
    PubSub,
    /// A function is scheduled to be executed regularly
    Timer,
    /// If none of the others apply
    Other,
}

impl Display for OpenTelemetryFaasTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenTelemetryFaasTrigger::Datasource => write!(f, "datasource"),
            OpenTelemetryFaasTrigger::Http => write!(f, "http"),
            OpenTelemetryFaasTrigger::PubSub => write!(f, "pubsub"),
            OpenTelemetryFaasTrigger::Timer => write!(f, "timer"),
            OpenTelemetryFaasTrigger::Other => write!(f, "other"),
        }
    }
}
