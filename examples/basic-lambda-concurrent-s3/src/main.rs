use aws_config::BehaviorVersion;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    bucket: String,
    key: String,
    body: String,
}

#[derive(Serialize)]
struct Response {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let s3_client = aws_sdk_s3::Client::new(&config);

    lambda_runtime::run_concurrent(service_fn(move |event: LambdaEvent<Request>| {
        let s3_client = s3_client.clone(); // cheap clone, no Arc needed
        async move {
            s3_client
                .put_object()
                .bucket(&event.payload.bucket)
                .key(&event.payload.key)
                .body(event.payload.body.into_bytes().into())
                .send()
                .await?;
            Ok::<Response, Error>(Response {
                message: "uploaded".into(),
            })
        }
    }))
    .await
}
