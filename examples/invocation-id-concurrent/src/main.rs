// This example requires the following input to succeed:
// { "command": "do something" }

use lambda_runtime::{service_fn, tracing, Diagnostic, Error, LambdaEvent};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Request {
    #[serde(rename = "command")]
    _command: String,
    sleep: u32,
}

#[derive(Serialize, Debug, PartialEq)]
struct Response {
    req_id: String,
    inv_id: Option<String>,
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


/**
 * Cross-wiring protection: duplicate request-id after timeout.

    Timeline:
        t=0: Invoke A starts, handler sleeps 7s
        t=5: A times out (timeout=5s). Batch 1 completes with timeout error.
        t=5: Invoke B starts (same request-id), handler sleeps 4s
        t=7: A's handler wakes up, posts stale /response/{same-id}
        t=9: B's handler wakes up, posts correct /response/{same-id}

        With invocation-id: A's stale post at t=7 gets 410 Gone. B responds at t=9 correctly.
        Without: A's stale response at t=7 is accepted for B (cross-wired).
 */

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
    if event.payload.sleep > 0 {
        tokio::time::sleep(tokio::time::Duration::from_secs(event.payload.sleep.into())).await;
    }

    let resp = Response {
        req_id: event.context.request_id,
        inv_id: event.context.invocation_id,
    };

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn handler_returns_request_and_invocation_ids() {
        let mut context = Context::default();
        context.request_id = "req-123".to_string();
        context.invocation_id = Some("inv-456".to_string());

        let payload = Request {
            _command: "test".to_string(),
            sleep: 0,
        };
        let event = LambdaEvent { payload, context };
        let result = my_handler(event).await.unwrap();

        assert_eq!(
            result,
            Response {
                req_id: "req-123".to_string(),
                inv_id: Some("inv-456".to_string()),
            }
        );
    }

    #[tokio::test]
    async fn handler_works_without_invocation_id() {
        let mut context = Context::default();
        context.request_id = "req-789".to_string();
        // invocation_id defaults to None

        let payload = Request {
            _command: "test".to_string(),
            sleep: 0,
        };
        let event = LambdaEvent { payload, context };
        let result = my_handler(event).await.unwrap();

        assert_eq!(
            result,
            Response {
                req_id: "req-789".to_string(),
                inv_id: None,
            }
        );
    }
}
