use lambda_runtime::{Error, LambdaEvent};
use serde_json::Value;
use std::time::Duration;

pub(crate) async fn function_handler(_event: LambdaEvent<Value>) -> Result<Value, Error> {
    tokio::time::sleep(Duration::from_secs(10)).await;
    Ok(Value::Null)
}
