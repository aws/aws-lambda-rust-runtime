use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, context) = event.into_parts();

    tracing::info!("Processing request with tenant ID: {:?}", context.tenant_id);

    let response = json!({
        "statusCode": 200,
        "body": json!({
            "message": "Tenant ID test successful",
            "request_id": context.request_id,
            "tenant_id": context.tenant_id,
            "input": event
        }).to_string()
    });

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::tracing::init_default_subscriber();
    lambda_runtime::run(service_fn(function_handler)).await
}
