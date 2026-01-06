use tower::{Layer, Service};
use tracing::{instrument::Instrumented, Instrument};

use crate::{Context, LambdaInvocation};
use lambda_runtime_api_client::BoxError;
use std::task;

/// Tower middleware to create a tracing span for invocations of the Lambda function.
#[derive(Default)]
pub struct TracingLayer {}

impl TracingLayer {
    /// Create a new tracing layer.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S> Layer<S> for TracingLayer {
    type Service = TracingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TracingService { inner }
    }
}

/// Tower service returned by [TracingLayer].
#[derive(Clone)]
pub struct TracingService<S> {
    inner: S,
}

impl<S> Service<LambdaInvocation> for TracingService<S>
where
    S: Service<LambdaInvocation, Response = (), Error = BoxError>,
{
    type Response = ();
    type Error = BoxError;
    type Future = Instrumented<S::Future>;

    fn poll_ready(&mut self, cx: &mut task::Context<'_>) -> task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        let span = request_span(&req.context);
        let future = {
            // Enter the span before calling the inner service
            // to ensure that it's assigned as parent of the inner spans.
            let _guard = span.enter();
            self.inner.call(req)
        };
        future.instrument(span)
    }
}

/* ------------------------------------------- UTILS ------------------------------------------- */

/// Creates a tracing span for a Lambda request with context information.
///
/// This function creates a span that includes the request ID and optionally
/// the X-Ray trace ID and tenant ID if they are available in the context.
pub fn request_span(ctx: &Context) -> tracing::Span {
    match (&ctx.xray_trace_id, &ctx.tenant_id) {
        (Some(trace_id), Some(tenant_id)) => {
            tracing::info_span!(
                "Lambda runtime invoke",
                requestId = &ctx.request_id,
                xrayTraceId = trace_id,
                tenantId = tenant_id
            )
        }
        (Some(trace_id), None) => {
            tracing::info_span!(
                "Lambda runtime invoke",
                requestId = &ctx.request_id,
                xrayTraceId = trace_id
            )
        }
        (None, Some(tenant_id)) => {
            tracing::info_span!(
                "Lambda runtime invoke",
                requestId = &ctx.request_id,
                tenantId = tenant_id
            )
        }
        (None, None) => {
            tracing::info_span!("Lambda runtime invoke", requestId = &ctx.request_id)
        }
    }
}
