use aws_lambda_events::{
    apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse},
    http::HeaderMap,
};
use lambda_runtime::{service_fn, tracing, Error, LambdaEvent};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    let func = service_fn(func);
    lambda_runtime::spawn_graceful_shutdown_handler(|| async move {}).await;
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(_event: LambdaEvent<ApiGatewayProxyRequest>) -> Result<ApiGatewayProxyResponse, Error> {
    let mut headers = HeaderMap::new();
    headers.insert("content-type", "text/html".parse().unwrap());
    let resp = {
        let mut response = ApiGatewayProxyResponse::default();
        response.status_code = 200;
        response.multi_value_headers = headers.clone();
        response.is_base64_encoded = false;
        response.body = Some("Hello world!".into());
        response.headers = headers;
        #[cfg(feature = "catch-all-fields")]
        {
            response.other = Default::default();
        }
        response
    };
    Ok(resp)
}
