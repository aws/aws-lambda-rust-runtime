use aws_lambda_events::http::{header::CONTENT_TYPE, HeaderMap, StatusCode};
use lambda_runtime::{
    service_fn,
    streaming::{channel, Body},
    tracing, Error, LambdaEvent, MetadataPrelude, StreamResponse,
};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    lambda_runtime::run(service_fn(func)).await
}

async fn func(_event: LambdaEvent<Value>) -> Result<StreamResponse<Body>, Error> {
    // A non-default status and a custom header prove that API Gateway applies the metadata prelude.
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, "text/plain".parse().unwrap());
    headers.insert("x-response-mode", "streaming".parse().unwrap());

    let (mut tx, rx) = channel();
    tokio::spawn(async move {
        for chunk in ["Hello", " from a", " streamed response"] {
            if tx.send_data(chunk.into()).await.is_err() {
                tracing::error!("response stream closed before all chunks were sent");
                return;
            }
        }
    });

    Ok(StreamResponse {
        metadata_prelude: MetadataPrelude {
            status_code: StatusCode::CREATED,
            headers,
            cookies: Vec::new(),
        },
        stream: rx,
    })
}
