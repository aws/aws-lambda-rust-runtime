// This example requires the following input to succeed:
// { "command": "do something" }

use lambda_runtime::{service_fn, tracing, Diagnostic, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    command: String,
}

#[derive(Serialize)]
struct Response {
    req_id: String,
    msg: String,
}

#[derive(Debug)]
struct HandlerError(String);

impl std::fmt::Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<HandlerError> for Diagnostic {
    fn from(e: HandlerError) -> Diagnostic {
        Diagnostic {
            error_type: "HandlerError".into(),
            error_message: e.0,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    let max_concurrency = std::env::var("AWS_LAMBDA_MAX_CONCURRENCY").unwrap_or_else(|_| "not set".to_string());
    tracing::info!(AWS_LAMBDA_MAX_CONCURRENCY = %max_concurrency, "starting concurrent handler");

    let func = service_fn(my_handler);
    if let Err(err) = lambda_runtime::run_concurrent(func).await {
        tracing::error!(error = %err, "run error");
        return Err(err);
    }
    Ok(())
}

pub(crate) async fn my_handler(event: LambdaEvent<Request>) -> Result<Response, HandlerError> {
    let command = event.payload.command;

    if command == "fail" {
        return Err(HandlerError("simulated handler error".into()));
    }

    let resp = Response {
        req_id: event.context.request_id,
        msg: format!("Command {command} executed."),
    };

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use crate::{my_handler, Request};
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn response_is_good_for_simple_input() {
        let id = "ID";

        let mut context = Context::default();
        context.request_id = id.to_string();

        let payload = Request {
            command: "X".to_string(),
        };
        let event = LambdaEvent { payload, context };

        let result = my_handler(event).await.unwrap();

        assert_eq!(result.msg, "Command X executed.");
        assert_eq!(result.req_id, id.to_string());
    }
}
