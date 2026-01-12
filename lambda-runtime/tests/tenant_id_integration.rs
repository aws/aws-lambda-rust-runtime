use http::HeaderMap;
use lambda_runtime::Context;
use std::sync::Arc;

#[test]
fn test_context_tenant_id_extraction() {
    let config = Arc::new(lambda_runtime::Config::default());

    // Test with tenant ID
    let mut headers = HeaderMap::new();
    headers.insert("lambda-runtime-aws-request-id", "test-id".parse().unwrap());
    headers.insert("lambda-runtime-deadline-ms", "123456789".parse().unwrap());
    headers.insert("lambda-runtime-aws-tenant-id", "my-tenant-123".parse().unwrap());

    let context = Context::new("test-id", config.clone(), &headers).unwrap();
    assert_eq!(context.tenant_id, Some("my-tenant-123".to_string()));

    // Test without tenant ID
    let mut headers = HeaderMap::new();
    headers.insert("lambda-runtime-aws-request-id", "test-id".parse().unwrap());
    headers.insert("lambda-runtime-deadline-ms", "123456789".parse().unwrap());

    let context = Context::new("test-id", config, &headers).unwrap();
    assert_eq!(context.tenant_id, None);
}

#[test]
fn test_context_tenant_id_with_special_characters() {
    let config = Arc::new(lambda_runtime::Config::default());

    // Test with tenant ID containing special characters
    let mut headers = HeaderMap::new();
    headers.insert("lambda-runtime-aws-request-id", "test-id".parse().unwrap());
    headers.insert("lambda-runtime-deadline-ms", "123456789".parse().unwrap());
    headers.insert(
        "lambda-runtime-aws-tenant-id",
        "tenant-with-dashes_and_underscores.123".parse().unwrap(),
    );

    let context = Context::new("test-id", config, &headers).unwrap();
    assert_eq!(
        context.tenant_id,
        Some("tenant-with-dashes_and_underscores.123".to_string())
    );
}

#[test]
fn test_context_tenant_id_empty_value() {
    let config = Arc::new(lambda_runtime::Config::default());

    // Test with empty tenant ID
    let mut headers = HeaderMap::new();
    headers.insert("lambda-runtime-aws-request-id", "test-id".parse().unwrap());
    headers.insert("lambda-runtime-deadline-ms", "123456789".parse().unwrap());
    headers.insert("lambda-runtime-aws-tenant-id", "".parse().unwrap());

    let context = Context::new("test-id", config, &headers).unwrap();
    assert_eq!(context.tenant_id, Some("".to_string()));
}

#[test]
fn test_trace_layer_request_span_creation() {
    use lambda_runtime::layers::trace::request_span;

    // Test with both trace ID and tenant ID
    let mut context = Context::default();
    context.request_id = "test-request".to_string();
    context.xray_trace_id = Some("trace-123".to_string());
    context.tenant_id = Some("tenant-456".to_string());

    let _span = request_span(&context);
    // Just verify the span can be created without panicking

    // Test with only trace ID
    let mut context = Context::default();
    context.request_id = "test-request".to_string();
    context.xray_trace_id = Some("trace-123".to_string());

    let _span = request_span(&context);

    // Test with only tenant ID
    let mut context = Context::default();
    context.request_id = "test-request".to_string();
    context.tenant_id = Some("tenant-only".to_string());

    let _span = request_span(&context);

    // Test with only request ID
    let mut context = Context::default();
    context.request_id = "test-request".to_string();

    let _span = request_span(&context);
}
