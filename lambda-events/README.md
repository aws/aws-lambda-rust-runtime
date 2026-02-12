# AWS Lambda Events

[![crates.io][crate-image]][crate-link]
[![Documentation][docs-image]][docs-link]

This crate provides strongly-typed [AWS Lambda event structs](https://docs.aws.amazon.com/lambda/latest/dg/invoking-lambda-function.html) in Rust.

## Installation

Add the dependency with Cargo: `cargo add aws_lambda_events`.

## Usage

The crate itself has no AWS Lambda handler logic and instead exists to serialize
and deserialize AWS Lambda events into strongly-typed Rust structs.

The types
defined in this crate are usually used with handlers / runtimes provided by the [official Rust runtime](https://github.com/aws/aws-lambda-rust-runtime).

For a list of supported AWS Lambda events and services, see [the crate reference documentation](https://docs.rs/aws_lambda_events).

## Conditional compilation of features

This crate divides all Lambda Events into features named after the service that the events are generated from. By default all events are enabled when you include this crate as a dependency to your project. If you only want to import specific events from this crate, you can disable the default features, and enable only the events that you need. This will make your project to compile a little bit faster, since rustc doesn't need to compile events that you're not going to use. Here's an example on how to do that:

```
cargo add aws_lambda_events --no-default-features --features apigw,alb
```

### Builder pattern support

The crate provides an optional `builders` feature that adds builder pattern support for event types. This enables type-safe, immutable construction of event responses with a clean, ergonomic API.

Enable the builders feature:

```
cargo add aws_lambda_events --features builders
```

Example using builders with API Gateway custom authorizers:

```rust
use aws_lambda_events::event::apigw::{
    ApiGatewayV2CustomAuthorizerSimpleResponse,
    ApiGatewayV2CustomAuthorizerV2Request,
};
use lambda_runtime::{Error, LambdaEvent};

// Context type without Default implementation
struct MyContext {
    user_id: String,
    permissions: Vec<String>,
}

async fn handler(
    event: LambdaEvent<ApiGatewayV2CustomAuthorizerV2Request>,
) -> Result<ApiGatewayV2CustomAuthorizerSimpleResponse<MyContext>, Error> {
    let context = MyContext {
        user_id: "user-123".to_string(),
        permissions: vec!["read".to_string()],
    };

    let response = ApiGatewayV2CustomAuthorizerSimpleResponse::builder()
        .is_authorized(true)
        .context(context)
        .build();

    Ok(response)
}
```

See the [examples directory](https://github.com/aws/aws-lambda-rust-runtime/tree/main/lambda-events/examples) for more builder pattern examples.

## History

The AWS Lambda Events crate was created by [Christian Legnitto](https://github.com/LegNeato). Without all his work and dedication, this project could have not been possible.

In 2023, the AWS Lambda Event crate was moved into this repository to continue its support for all AWS customers that use Rust on AWS Lambda.

[//]: # 'badges'
[crate-image]: https://img.shields.io/crates/v/aws_lambda_events.svg
[crate-link]: https://crates.io/crates/aws_lambda_events
[docs-image]: https://docs.rs/aws_lambda_events/badge.svg
[docs-link]: https://docs.rs/aws_lambda_events