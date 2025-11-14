# lambda-http for AWS Lambda in Rust

[![Docs](https://docs.rs/lambda_http/badge.svg)](https://docs.rs/lambda_http)

**`lambda-http`** is an abstraction that takes payloads from different services and turns them into http objects, making it easy to write API Gateway proxy event focused Lambda functions in Rust.

lambda-http handler is made of:

* `Request` - Represents an HTTP request
* `IntoResponse` - Future that will convert an [`IntoResponse`] into an actual [`LambdaResponse`]

We are able to handle requests from:

* [API Gateway](https://docs.aws.amazon.com/apigateway/latest/developerguide/welcome.html) REST, HTTP and WebSockets API lambda integrations
* AWS [ALB](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/introduction.html)
* AWS [Lambda function URLs](https://docs.aws.amazon.com/lambda/latest/dg/lambda-urls.html)

Thanks to the `Request` type we can seamlessly handle proxy integrations without the worry to specify the specific service type.

There is also an extension for `lambda_http::Request` structs that provides access to [API gateway](https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format) and [ALB](https://docs.aws.amazon.com/elasticloadbalancing/latest/application/lambda-functions.html) features.

For example some handy extensions:

* `query_string_parameters` - Return pre-parsed http query string parameters, parameters provided after the `?` portion of a url associated with the request
* `path_parameters` - Return pre-extracted path parameters, parameter provided in url placeholders `/foo/{bar}/baz/{qux}` associated with the request
* `lambda_context` - Return the Lambda context for the invocation; see the [runtime docs](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html#runtimes-api-next)
* `request_context` - Return the ALB/API Gateway request context
* payload - Return the Result of a payload parsed into a type that implements `serde::Deserialize`

See the `lambda_http::RequestPayloadExt` and `lambda_http::RequestExt` traits for more extensions.

## Examples

Here you will find a few examples to handle basic scenarios:

* Reading a JSON from a body and deserialize into a structure
* Reading query string parameters
* Lambda Request Authorizer
* Passing the Lambda execution context initialization to the handler

### Reading a JSON from a body and deserialize into a structure

The code below creates a simple API Gateway proxy (HTTP, REST) that accepts in input a JSON payload.

```rust
use lambda_http::{run, http::{StatusCode, Response}, service_fn, Error, IntoResponse, Request, RequestPayloadExt};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .init();

    run(service_fn(function_handler)).await
}

pub async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    let body = event.payload::<MyPayload>()?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json!({
            "message": "Hello World",
            "payload": body,
          }).to_string())
        .map_err(Box::new)?;

    Ok(response)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MyPayload {
    pub prop1: String,
    pub prop2: String,
}
```

### Reading query string parameters

```rust
use lambda_http::{run, http::{StatusCode, Response}, service_fn, Error, RequestExt, IntoResponse, Request};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .init();

    run(service_fn(function_handler)).await
}

pub async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    let name = event.query_string_parameters_ref()
        .and_then(|params| params.first("name"))
        .unwrap_or_else(|| "stranger")
        .to_string();

    // Represents an HTTP response
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json!({
            "message": format!("Hello, {}!", name),
          }).to_string())
        .map_err(Box::new)?;

    Ok(response)
}
```

### Lambda Request Authorizer

Because **`lambda-http`** is an abstraction, we cannot use it for the Lambda Request Authorizer case.
If you remove the abstraction, you need to handle the request/response for your service.

```rust
use aws_lambda_events::apigw::{
    ApiGatewayCustomAuthorizerRequestTypeRequest, ApiGatewayCustomAuthorizerResponse, ApiGatewayCustomAuthorizerPolicy, IamPolicyStatement,
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .init();

    run(service_fn(function_handler)).await
}

pub async fn function_handler(event: LambdaEvent<ApiGatewayCustomAuthorizerRequestTypeRequest>) -> Result<ApiGatewayCustomAuthorizerResponse, Error> {
    // do something with the event payload
    let method_arn = event.payload.method_arn.unwrap();
    // for example we could use the authorization header
    if let Some(token) = event.payload.headers.get("authorization") {
        // do something

        return Ok(custom_authorizer_response(
            "ALLOW",
            "some_principal",
            &method_arn,
        ));
    }

    Ok(custom_authorizer_response(
      &"DENY".to_string(),
      "",
      &method_arn))
}

pub fn custom_authorizer_response(effect: &str, principal: &str, method_arn: &str) -> ApiGatewayCustomAuthorizerResponse {
    let stmt = IamPolicyStatement {
        action: vec!["execute-api:Invoke".to_string()],
        resource: vec![method_arn.to_owned()],
        effect: Some(effect.to_owned()),
    };
    let policy = ApiGatewayCustomAuthorizerPolicy {
        version: Some("2012-10-17".to_string()),
        statement: vec![stmt],
    };
    ApiGatewayCustomAuthorizerResponse {
        principal_id: Some(principal.to_owned()),
        policy_document: policy,
        context: json!({ "email": principal }), // https://github.com/aws/aws-lambda-rust-runtime/discussions/548
        usage_identifier_key: None,
    }
}
```

### Passing the Lambda execution context initialization to the handler

One of the [best practices](https://docs.aws.amazon.com/lambda/latest/dg/best-practices.html) is to take advantage of execution environment reuse to improve the performance of your function. Initialize SDK clients and database connections outside the function handler. Subsequent invocations processed by the same instance of your function can reuse these resources. This saves cost by reducing function run time.

```rust
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::Utc;
use lambda_http::{run, http::{StatusCode, Response}, service_fn, Error, RequestExt, IntoResponse, Request};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .without_time()
        .with_max_level(tracing::Level::INFO)
        .init();

    let config = aws_config::from_env()
        .load()
        .await;

    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

    run(service_fn(|event: Request| function_handler(&dynamodb_client, event))).await
}

pub async fn function_handler(dynamodb_client: &aws_sdk_dynamodb::Client, event: Request) -> Result<impl IntoResponse, Error> {
    let table = std::env::var("TABLE_NAME").expect("TABLE_NAME must be set");

    let name = event.query_string_parameters_ref()
        .and_then(|params| params.first("name"))
        .unwrap_or_else(|| "stranger")
        .to_string();

    dynamodb_client
        .put_item()
        .table_name(table)
        .item("ID", AttributeValue::S(Utc::now().timestamp().to_string()))
        .item("name", AttributeValue::S(name.to_owned()))
        .send()
        .await?;

    // Represents an HTTP response
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(json!({
            "message": format!("Hello, {}!", name),
          }).to_string())
        .map_err(Box::new)?;

    Ok(response)
}
```

## Integration with API Gateway stages

When you integrate HTTP Lambda functions with API Gateway stages, the path received in the request will include the stage as the first segment, for example `/production/api/v1`, where `production` is the API Gateway stage.

If you don't want to receive the stage as part of the path, you can set the environment variable `AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH` to `true`, either in your Lambda function configuration, or inside the `main` Rust function. Following the previous example, when this environment variable is present, the path that the function receives is `/api/v1`, eliminating the stage from the first segment.

## Feature flags

`lambda_http` is a wrapper for HTTP events coming from three different services, Amazon Load Balancer (ALB), Amazon Api Gateway (APIGW), and AWS Lambda Function URLs. Amazon Api Gateway can also send events from three different endpoints, REST APIs, HTTP APIs, and WebSockets. `lambda_http` transforms events from all these sources into native `http::Request` objects, so you can incorporate Rust HTTP semantics into your Lambda functions.

By default, `lambda_http` compiles your function to support any of those services. This increases the compile time of your function because we have to generate code for all the sources. In reality, you'll usually put a Lambda function only behind one of those sources. You can choose which source to generate code for with feature flags.

The available features flags for `lambda_http` are the following:

- `alb`: for events coming from [Amazon Elastic Load Balancer](https://aws.amazon.com/elasticloadbalancing/).
- `apigw_rest`: for events coming from [Amazon API Gateway Rest APIs](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-rest-api.html).
- `apigw_http`: for events coming from [Amazon API Gateway HTTP APIs](https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api.html) and [AWS Lambda Function URLs](https://docs.aws.amazon.com/lambda/latest/dg/lambda-urls.html).
- `apigw_websockets`: for events coming from [Amazon API Gateway WebSockets](https://docs.aws.amazon.com/apigateway/latest/developerguide/apigateway-websocket-api.html).

If you only want to support one of these sources, you can disable the default features, and enable only the source that you care about in your package's `Cargo.toml` file. Substitute the dependency line for `lambda_http` for the snippet below, changing the feature that you want to enable:

```toml
[dependencies.lambda_http]
version = "0.5.3"
default-features = false
features = ["apigw_rest"]
```
