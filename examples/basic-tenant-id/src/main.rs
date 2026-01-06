use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, context) = event.into_parts();
    
    // Access tenant ID from context
    let tenant_info = match &context.tenant_id {
        Some(tenant_id) => format!("Processing request for tenant: {}", tenant_id),
        None => "No tenant ID provided".to_string(),
    };
    
    tracing::info!("Request ID: {}", context.request_id);
    tracing::info!("Tenant info: {}", tenant_info);
    
    // Include tenant ID in response
    let response = json!({
        "message": "Hello from Lambda!",
        "request_id": context.request_id,
        "tenant_id": context.tenant_id,
        "input": event
    });

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();

    lambda_runtime::run(service_fn(function_handler)).await
}
