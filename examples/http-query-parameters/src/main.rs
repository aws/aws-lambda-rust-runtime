use lambda_http::{run, service_fn, tracing, Error, IntoResponse, Request, RequestExt, Response};

/// This is the main body for the function.
/// Write your code inside it.
/// You can see more examples in Runtime's repository:
/// - <https://github.com/aws/aws-lambda-rust-runtime/tree/main/examples>
async fn function_handler(event: Request) -> Result<impl IntoResponse, Error> {
    // Extract some useful information from the request
    Ok(
        match event
            .query_string_parameters_ref()
            .and_then(|params| params.first("first_name"))
        {
            Some(first_name) => format!("Hello, {first_name}!").into_response().await,
            None => Response::builder()
                .status(400)
                .body("Empty first name".into())
                .expect("failed to render response"),
        },
    )
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
