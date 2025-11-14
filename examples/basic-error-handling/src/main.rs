/// See <https://github.com/aws/aws-lambda-rust-runtime> for more info on Rust runtime for AWS Lambda
use lambda_runtime::{service_fn, tracing, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;

/// A simple Lambda request structure with just one field
/// that tells the Lambda what is expected of it.
#[derive(Deserialize)]
struct Request {
    event_type: EventType,
}

/// Event types that tell our Lambda what to do do.
#[derive(Deserialize, Eq, PartialEq)]
enum EventType {
    Response,
    ExternalError,
    SimpleError,
    CustomError,
    Panic,
}

/// A simple Lambda response structure.
#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[derive(Debug, Serialize)]
struct CustomError {
    is_authenticated: bool,
    req_id: String,
    msg: String,
}

impl std::error::Error for CustomError {
    // this implementation required `Debug` and `Display` traits
}

impl std::fmt::Display for CustomError {
    /// Display the error struct as a JSON string
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_as_json = json!(self).to_string();
        write!(f, "{err_as_json}")
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    // call the actual handler of the request
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

/// The actual handler of the Lambda request.
pub(crate) async fn func(event: LambdaEvent<Request>) -> Result<Response, Error> {
    let (event, ctx) = event.into_parts();

    // check what action was requested
    match event.event_type {
        EventType::SimpleError => {
            // generate a simple text message error using `simple_error` crate
            Err(Box::new(simple_error::SimpleError::new("A simple error as requested!")))
        }
        EventType::CustomError => {
            // generate a custom error using our own structure
            let cust_err = CustomError {
                is_authenticated: ctx.identity.is_some(),
                req_id: ctx.request_id,
                msg: "A custom error as requested!".into(),
            };
            Err(Box::new(cust_err))
        }
        EventType::ExternalError => {
            // try to open a non-existent file to get an error and propagate it with `?`
            let _file = File::open("non-existent-file.txt")?;

            // it should never execute past the above line
            unreachable!();
        }
        EventType::Panic => {
            panic!();
        }
        EventType::Response => {
            // generate and return an OK response in JSON format
            let resp = Response {
                req_id: ctx.request_id,
                msg: "OK".into(),
            };

            Ok(resp)
        }
    }
}
