# Rust Runtime for AWS Lambda

[![Build Status](https://github.com/awslabs/aws-lambda-rust-runtime/actions/workflows/check-examples.yml/badge.svg)](https://github.com/awslabs/aws-lambda-rust-runtime/actions)

This package makes it easy to run AWS Lambda Functions written in Rust. This workspace includes multiple crates:

- [![Docs](https://docs.rs/lambda_runtime/badge.svg)](https://docs.rs/lambda_runtime) **`lambda-runtime`** is a library that provides a Lambda runtime for applications written in Rust.
- [![Docs](https://docs.rs/lambda_http/badge.svg)](https://docs.rs/lambda_http) **`lambda-http`** is a library that makes it easy to write API Gateway proxy event focused Lambda functions in Rust.
- [![Docs](https://docs.rs/lambda-extension/badge.svg)](https://docs.rs/lambda-extension) **`lambda-extension`** is a library that makes it easy to write Lambda Runtime Extensions in Rust.
- [![Docs](https://docs.rs/aws_lambda_events/badge.svg)](https://docs.rs/aws_lambda_events) **`lambda-events`** is a library with strongly-typed Lambda event structs in Rust.
- [![Docs](https://docs.rs/lambda_runtime_api_client/badge.svg)](https://docs.rs/lambda_runtime_api_client) **`lambda-runtime-api-client`** is a shared library between the lambda runtime and lambda extension libraries that includes a common API client to talk with the AWS Lambda Runtime API.

The Rust runtime client is an experimental package. It is subject to change and intended only for evaluation purposes.

## Getting started

The easiest way to start writing Lambda functions with Rust is by using [Cargo Lambda](https://www.cargo-lambda.info/), a related project. Cargo Lambda is a Cargo plugin, or subcommand, that provides several commands to help you in your journey with Rust on AWS Lambda.

The preferred way to install Cargo Lambda is by using a package manager.

1- Use Homebrew on [MacOS](https://brew.sh/):

```bash
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda
```

2- Use [Scoop](https://scoop.sh/) on Windows:

```bash
scoop bucket add cargo-lambda https://github.com/cargo-lambda/scoop-cargo-lambda
scoop install cargo-lambda/cargo-lambda
```

Or PiP on any system with Python 3 installed:

```bash
pip3 install cargo-lambda
```
Alternative, install the pip package as an executable using [uv](https://docs.astral.sh/uv/)

```bash
uv tool install cargo-lambda
```

See other installation options in [the Cargo Lambda documentation](https://www.cargo-lambda.info/guide/installation.html).

## Your first function

To create your first function, run Cargo Lambda with the [subcommand `new`](https://www.cargo-lambda.info/commands/new.html). This command will generate a Rust package with the initial source code for your function:

```bash
cargo lambda new YOUR_FUNCTION_NAME
```

### Example function

If you'd like to manually create your first function, the code below shows you a simple function that receives an event with a `firstName` field and returns a message to the caller.

```rust,no_run
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();
    let first_name = event["firstName"].as_str().unwrap_or("world");

    Ok(json!({ "message": format!("Hello, {}!", first_name) }))
}
```

## Understanding Lambda errors

when a function invocation fails, AWS Lambda expects you to return an object that can be serialized into JSON structure with the error information. This structure is represented in the following example:

```json
{
  "error_type": "the type of error raised",
  "error_message": "a string description of the error"
}
```

The Rust Runtime for Lambda uses a struct called `Diagnostic` to represent function errors internally. The runtime implements the conversion of several general error types, like `std::error::Error`, into `Diagnostic`. For these general implementations, the `error_type` is the name of the value type returned by your function. For example, if your function returns `lambda_runtime::Error`, the `error_type` will be something like `alloc::boxed::Box<dyn core::error::Error + core::marker::Send + core::marker::Sync>`, which is not very descriptive.

### Implement your own Diagnostic

To get more descriptive `error_type` fields, you can implement `From` for your error type. That gives you full control on what the `error_type` is:

```rust
use lambda_runtime::{Diagnostic, Error, LambdaEvent};

#[derive(Debug)]
struct ErrorResponse(&'static str);

impl From<ErrorResponse> for Diagnostic {
    fn from(error: ErrorResponse) -> Diagnostic {
        Diagnostic {
            error_type: "MyErrorType".into(),
            error_message: error.0.to_string(),
        }
    }
}

async fn handler(_event: LambdaEvent<()>) -> Result<(), ErrorResponse> {
  Err(ErrorResponse("this is an error response"))
}
```

We recommend you to use the [thiserror crate](https://crates.io/crates/thiserror) to declare your errors. You can see an example on how to integrate `thiserror` with the Runtime's diagnostics in our [example repository](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples/basic-error-thiserror)

### Anyhow, Eyre, and Miette

Popular error crates like Anyhow, Eyre, and Miette provide their own error types that encapsulate other errors. There is no direct transformation of those errors into `Diagnostic`, but we provide feature flags for each one of those crates to help you integrate them with your Lambda functions.

If you enable the features `anyhow`, `eyre`, or `miette` in the `lambda_runtime` dependency of your package. The error types provided by those crates can have blanket transformations into `Diagnostic`. These features expose an `From<T> for Diagnostic` implementation that transforms those error types into a `Diagnostic`. This is an example that transforms an `anyhow::Error` into a `Diagnostic`:

```rust
use lambda_runtime::{Diagnostic, LambdaEvent};

async fn handler(_event: LambdaEvent<Request>) -> Result<(), Diagnostic> {
  Err(anyhow::anyhow!("this is an error").into())
}
```

You can see more examples on how to use these error crates in our [example repository](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples/basic-error-error-crates-integration). 

### Graceful shutdown

`lambda_runtime` offers a helper to simplify configuring graceful shutdown signal handling, `spawn_graceful_shutdown_handler()`. This requires the `graceful-shutdown` feature flag and only supports Unix systems.

You can use it by passing a `FnOnce` closure that returns an async block. That async block will be executed
when the function receives a `SIGTERM` or `SIGKILL`.

Note that this helper is opinionated in a number of ways. Most notably:
1. It spawns a task to drive your signal handlers
2. It registers a 'no-op' extension in order to enable graceful shutdown signals
3. It panics on unrecoverable errors

If you prefer to fine-tune the behavior, refer to the implementation of `spawn_graceful_shutdown_handler()` as a starting point for your own.

For more information on graceful shutdown handling in AWS Lambda, see: [aws-samples/graceful-shutdown-with-aws-lambda](https://github.com/aws-samples/graceful-shutdown-with-aws-lambda).

Complete example (cleaning up a non-blocking tracing writer):

```rust,no_run
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);

    let (writer, log_guard) = tracing_appender::non_blocking(std::io::stdout());
    lambda_runtime::tracing::init_default_subscriber_with_writer(writer);

    let shutdown_hook = || async move {
      std::mem::drop(log_guard);
    };
    lambda_runtime::spawn_graceful_shutdown_handler(shutdown_hook).await;

    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();
    let first_name = event["firstName"].as_str().unwrap_or("world");

    Ok(json!({ "message": format!("Hello, {}!", first_name) }))
}
```

## Building and deploying your Lambda functions

If you already have Cargo Lambda installed in your machine, run the next command to build your function:

```bash
cargo lambda build --release
```

There are other ways of building your function: manually with the AWS CLI, with [AWS SAM](https://github.com/aws/aws-sam-cli), and with the [Serverless framework](https://serverless.com/framework/).

### 1. Cross-compiling your Lambda functions

By default, Cargo Lambda builds your functions to run on x86_64 architectures. If you'd like to use a different architecture, use the options described below.

#### 1.1. Build your Lambda functions

__Amazon Linux 2023__

We recommend you to use the Amazon Linux 2023  (such as `provided.al2023`) because it includes a newer version of GLIBC, which many Rust programs depend on. To build your Lambda functions for Amazon Linux 2023 runtimes, run:

```bash
cargo lambda build --release --arm64
```

### 2. Deploying the binary to AWS Lambda

For [a custom runtime](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-custom.html), AWS Lambda looks for an executable called `bootstrap` in the deployment package zip. Rename the generated executable to `bootstrap` and add it to a zip archive.

You can find the `bootstrap` binary for your function under the `target/lambda` directory.

#### 2.1. Deploying with Cargo Lambda

Once you've built your code with one of the options described earlier, use the `deploy` subcommand to upload your function to AWS:

```bash
cargo lambda deploy
```

> **Warning**
> Make sure to replace the execution role with an existing role in your account!

This command will create a Lambda function with the same name of your rust package. You can change the name
of the function by adding the argument at the end of the command:

```bash
cargo lambda deploy my-first-lambda-function
```

> **Note**
> See other deployment options in [the Cargo Lambda documentation](https://www.cargo-lambda.info/commands/deploy.html).

You can test the function with the [invoke subcommand](https://www.cargo-lambda.info/commands/invoke.html):

```bash
cargo lambda invoke --remote \
  --data-ascii '{"command": "hi"}' \
  --output-format json \
  my-first-lambda-function
```

> **Note**
> CLI commands in the examples use Linux/MacOS syntax. For different shells like Windows CMD or PowerShell, modify syntax when using nested quotation marks like `'{"command": "hi"}'`. Escaping with a backslash may be necessary. See [AWS CLI Reference](https://docs.amazonaws.cn/en_us/cli/latest/userguide/cli-usage-parameters-quoting-strings.html#cli-usage-parameters-quoting-strings-containing) for more information.

#### 2.2. Deploying with the AWS CLI

You can also use the AWS CLI to deploy your Rust functions. First, you will need to create a ZIP archive of your  function. Cargo Lambda can do that for you automatically when it builds your binary if you add the `output-format` flag:

```bash
cargo lambda build --release --arm64 --output-format zip
```

You can find the resulting zip file in `target/lambda/YOUR_PACKAGE/bootstrap.zip`. Use that file path to deploy your function with the [AWS CLI](https://aws.amazon.com/cli/):

```bash
$ aws lambda create-function --function-name rustTest \
  --handler bootstrap \
  --zip-file fileb://./target/lambda/basic/bootstrap.zip \
  --runtime provided.al2023 \ # Change this to provided.al2 if you would like to use Amazon Linux 2
  --role arn:aws:iam::XXXXXXXXXXXXX:role/your_lambda_execution_role \
  --environment Variables={RUST_BACKTRACE=1} \
  --tracing-config Mode=Active
```

> **Warning**
> Make sure to replace the execution role with an existing role in your account!

You can now test the function using the AWS CLI or the AWS Lambda console

```bash
$ aws lambda invoke
  --cli-binary-format raw-in-base64-out \
  --function-name rustTest \
  --payload '{"command": "Say Hi!"}' \
  output.json
$ cat output.json  # Prints: {"msg": "Command Say Hi! executed."}
```

> **Note** 
> `--cli-binary-format raw-in-base64-out` is a required argument when using the AWS CLI version 2. [More Information](https://docs.aws.amazon.com/cli/latest/userguide/cliv2-migration.html#cliv2-migration-binaryparam)

#### 2.3. AWS Serverless Application Model (SAM)

You can use Lambda functions built in Rust with the [AWS Serverless Application Model (SAM)](https://aws.amazon.com/serverless/sam/). To do so, you will need to install the [AWS SAM CLI](https://github.com/aws/aws-sam-cli), which will help you package and deploy your Lambda functions in your AWS account.

You will need to create a `template.yaml` file containing your desired infrastructure in YAML. Here is an example with a single Lambda function:

```yaml
AWSTemplateFormatVersion: '2010-09-09'
Transform: AWS::Serverless-2016-10-31

Resources:
  HelloWorldFunction:
    Type: AWS::Serverless::Function
    Properties:
      MemorySize: 128
      Architectures: ["arm64"]
      Handler: bootstrap
      Runtime: provided.al2023
      Timeout: 5
      CodeUri: target/lambda/basic/

Outputs:
  FunctionName:
    Value: !Ref HelloWorldFunction
    Description: Name of the Lambda function
```

You can then deploy your Lambda function using the AWS SAM CLI:

```bash
sam deploy --guided
```

At the end, `sam` will output the actual Lambda function name. You can use this name to invoke your function:

```bash
$ aws lambda invoke
  --cli-binary-format raw-in-base64-out \
  --function-name HelloWorldFunction-XXXXXXXX \ # Replace with the actual function name
  --payload '{"command": "Say Hi!"}' \
  output.json
$ cat output.json  # Prints: {"msg": "Command Say Hi! executed."}
```

## Local development and testing

### Testing your code with unit and integration tests

AWS Lambda events are plain structures deserialized from JSON objects.
If your function handler uses the standard runtime, you can use `serde` to deserialize
your text fixtures into the structures, and call your handler directly:

```rust,no_run
#[test]
fn test_my_lambda_handler() {
  let input = serde_json::from_str("{\"command\": \"Say Hi!\"}").expect("failed to parse event");
  let context = lambda_runtime::Context::default();

  let event = lambda_runtime::LambdaEvent::new(input, context);

  my_lambda_handler(event).await.expect("failed to handle event");
}
```

If you're using `lambda_http` to receive HTTP events, you can also create `http_lambda::Request`
structures from plain text fixtures:

```rust,no_run
#[test]
fn test_my_lambda_handler() {
  let input = include_str!("apigw_proxy_request.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let response = my_lambda_handler(request).await.expect("failed to handle request");
}
```

### Local dev server with Cargo Lambda

[Cargo Lambda](https://www.cargo-lambda.info) provides a local server that emulates the AWS Lambda control plane. This server works on Windows, Linux, and MacOS. In the root of your Lambda project. You can run the following subcommand to compile your function(s) and start the server.

```bash
cargo lambda watch
```

Now you can use the `cargo lambda invoke` to send requests to your function. For example:

```bash
cargo lambda invoke <lambda-function-name> --data-ascii '{ "command": "hi" }'
```

Running the command on a HTTP function (Function URL, API Gateway, etc) will require you to use the appropriate scheme. You can find examples of these schemes [here](https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/lambda-http/tests/data). Otherwise, you will be presented with the following error.

```rust,no_run
Error: serde_json::error::Error

  × data did not match any variant of untagged enum LambdaRequest
```

An simpler alternative is to cURL the following endpoint based on the address and port you defined. For example:

```bash
curl -v -X POST \
  'http://127.0.0.1:9000/lambda-url/<lambda-function-name>/' \
  -H 'content-type: application/json' \
  -d '{ "command": "hi" }'
```

> **Warning** 
> Do not remove the `content-type` header. It is necessary to instruct the function how to deserialize the request body.

You can read more about how [cargo lambda watch](https://www.cargo-lambda.info/commands/watch.html) and [cargo lambda invoke](https://www.cargo-lambda.info/commands/invoke.html) work on the project's [documentation page](https://www.cargo-lambda.info).

### Local testing with Runtime Interface Emulator (RIE)

For testing with the official AWS Lambda Runtime Interface Emulator, use the provided RIE testing infrastructure:

```bash
make test-rie
```

By default, this uses the `basic-lambda` example. To test a different example:

```bash
make test-rie EXAMPLE=basic-sqs
make test-rie EXAMPLE=http-basic-lambda
```

This command will:
1. Build a Docker image with Rust toolchain and RIE
2. Compile the specified example inside the Linux container
3. Start the RIE container on port 9000
4. Display the appropriate curl command for testing

Different examples expect different payload formats. Check the example's source code in `examples/EXAMPLE_NAME/src/main.rs`

This provides automated testing with Docker and RIE, ensuring your Lambda functions work in a Linux environment identical to AWS Lambda.

### Lambda Debug Proxy

Lambdas can be run and debugged locally using a special [Lambda debug proxy](https://github.com/rimutaka/lambda-debug-proxy) (a non-AWS repo maintained by @rimutaka), which is a Lambda function that forwards incoming requests to one AWS SQS queue and reads responses from another queue. A local proxy running on your development computer reads the queue, calls your Lambda locally and sends back the response. This approach allows debugging of Lambda functions locally while being part of your AWS workflow. The Lambda handler code does not need to be modified between the local and AWS versions.

## Tracing and Logging

The Rust Runtime for Lambda integrates with the [Tracing](https://tracing.rs) libraries to provide tracing and logging.

By default, the runtime emits `tracing` events that you can collect via `tracing-subscriber`. It also enabled a feature called `tracing` that exposes a default subscriber with sensible options to send logging information to AWS CloudWatch. Follow the next example that shows how to enable the default subscriber:

```rust
use lambda_runtime::{run, service_fn, tracing, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    run(service_fn(|event| tracing::info!(?event))).await
}
```

The subscriber uses `RUST_LOG` environment variable to determine the log level for your function. It also uses [Lambda's advanced logging controls](https://aws.amazon.com/blogs/compute/introducing-advanced-logging-controls-for-aws-lambda-functions/), if configured.

By default, the log level to emit events is `INFO`. Log at `TRACE` level for more detail, including a dump of the raw payload.

## AWS event objects

This project includes Lambda event struct definitions, [`aws_lambda_events`](https://crates.io/crates/aws_lambda_events). This crate can be leveraged to provide strongly-typed Lambda event structs. You can create your own custom event objects and their corresponding structs as well.

### Custom event objects

To serialize and deserialize events and responses, we suggest using the [`serde`](https://github.com/serde-rs/serde) library. To receive custom events, annotate your structure with Serde's macros:

```rust,no_run
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::error::Error;

#[derive(Serialize, Deserialize)]
pub struct NewIceCreamEvent {
  pub flavors: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct NewIceCreamResponse {
  pub flavors_added_count: usize,
}

fn main() -> Result<(), Box<Error>> {
    let flavors = json!({
      "flavors": [
        "Nocciola",
        "抹茶",
        "आम"
      ]
    });

    let event: NewIceCreamEvent = serde_json::from_value(flavors)?;
    let response = NewIceCreamResponse {
        flavors_added_count: event.flavors.len(),
    };
    serde_json::to_string(&response)?;

    Ok(())
}
```

## Supported Rust Versions (MSRV)

The AWS Lambda Rust Runtime requires a minimum of Rust 1.82.0, and is not guaranteed to build on compiler versions earlier than that.

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.
