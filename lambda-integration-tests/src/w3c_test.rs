use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

async fn function_handler(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (_event, context) = event.into_parts();

    let w3c_fields = context.w3c();
    tracing::info!("w3c fields observed on context: {:?}", w3c_fields);

    let client_context_has_custom = context
        .client_context
        .as_ref()
        .map(|cc| !cc.custom.is_empty())
        .unwrap_or(false);

    let response = json!({
        "statusCode": 200,
        "body": json!({
            "message": "W3C test successful",
            "request_id": context.request_id,
            "w3c": w3c_fields,
            "has_client_context": context.client_context.is_some(),
            "client_context_has_custom": client_context_has_custom,
        }).to_string()
    });

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::tracing::init_default_subscriber();
    lambda_runtime::run(service_fn(function_handler)).await
}
