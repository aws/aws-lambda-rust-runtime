use aws_lambda_events::{
    event::sqs::SqsEventObj,
    sqs::{BatchItemFailure, SqsBatchResponse, SqsMessageObj},
};
use futures::Future;
use lambda_runtime::{
    run, service_fn,
    tracing::{self, Instrument},
    Error, LambdaEvent,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// [To customize] Your object definition, sent to the SQS queue triggering this lambda.
#[derive(Deserialize, Serialize)]
struct Data {
    text: String,
}

/// [To customize] Your buisness logic to handle the payload of one SQS message.
async fn data_handler(data: Data) -> Result<(), Error> {
    // Some processing
    tracing::info!(text = ?data.text, "processing data");
    // simulate error
    if data.text == "bad request" {
        Err("Processing error".into())
    } else {
        Ok(())
    }
}

/// Main function for the lambda executable.
#[tokio::main]
async fn main() -> Result<(), Error> {
    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();

    run_sqs_partial_batch_failure(data_handler).await
}

/// This function will handle the message batches from SQS.
/// It calls the provided user function `f` on every message concurrently and reports to SQS
/// which message failed to be processed so that only those are retried.
///
/// Important note: your lambda sqs trigger *needs* to be configured with partial batch response support
/// with the ` ReportBatchItemFailures` flag set to true, otherwise failed message will be dropped,
/// for more details see:
/// <https://docs.aws.amazon.com/lambda/latest/dg/with-sqs.html#services-sqs-batchfailurereporting>
///
///
/// Note that if you are looking for parallel processing (multithread) instead of concurrent processing,
/// you can do so by spawning a task inside your function `f`.
async fn run_sqs_partial_batch_failure<T, D, R>(f: T) -> Result<(), Error>
where
    T: Fn(D) -> R,
    D: DeserializeOwned,
    R: Future<Output = Result<(), Error>>,
{
    run(service_fn(|e| batch_handler(&f, e))).await
}

/// Helper function to lift the user provided `f` function from message to batch of messages.
/// See `run_sqs` for the easier function to use.
async fn batch_handler<T, D, F>(
    f: T,
    event: LambdaEvent<SqsEventObj<serde_json::Value>>,
) -> Result<SqsBatchResponse, Error>
where
    T: Fn(D) -> F,
    F: Future<Output = Result<(), Error>>,
    D: DeserializeOwned,
{
    tracing::trace!("Handling batch size {}", event.payload.records.len());
    let create_task = |msg| {
        // We need to keep the message_id to report failures to SQS
        let SqsMessageObj { message_id, body, .. } = msg;
        let span = tracing::span!(tracing::Level::INFO, "Handling SQS msg", message_id);
        let task = async {
            //TODO catch panics like the `run` function from lambda_runtime
            f(serde_json::from_value(body)?).await
        }
        .instrument(span);
        (message_id.unwrap_or_default(), task)
    };
    let (ids, tasks): (Vec<_>, Vec<_>) = event.payload.records.into_iter().map(create_task).unzip();
    let results = futures::future::join_all(tasks).await; // Run tasks concurrently
    let failure_items = ids
        .into_iter()
        .zip(results)
        .filter_map(
            // Only keep the message_id of failed tasks
            |(id, res)| match res {
                Ok(()) => None,
                Err(err) => {
                    tracing::error!("Failed to process msg {id}, {err}");
                    Some(id)
                }
            },
        )
        .map(|id| {
            let mut failure_item =  BatchItemFailure::default();
            failure_item.item_identifier = id;
            failure_item
        })
        .collect();

    Ok({
        let mut response = SqsBatchResponse::default();
        response.batch_item_failures = failure_items;
        response
    })
}

#[cfg(test)]
mod test {
    use lambda_runtime::Context;

    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct UserData {
        should_error: bool,
    }
    async fn user_fn(data: UserData) -> Result<(), Error> {
        if data.should_error {
            Err("Processing Error".into())
        } else {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test() {
        let msg_to_fail: SqsMessageObj<serde_json::Value> = serde_json::from_str(
            r#"{
                "messageId": "1",
                "body": "{\"should_error\": true}"
            }"#,
        )
        .unwrap();
        let msg_to_succeed: SqsMessageObj<serde_json::Value> = serde_json::from_str(
            r#"{
                "messageId": "0",
                "body": "{\"should_error\" : false}"
            }"#,
        )
        .unwrap();

        let lambda_event = LambdaEvent {
            payload: {
                let mut event_object = SqsEventObj::default();
                event_object.records = vec![msg_to_fail, msg_to_succeed];
                event_object
            },
            context: Context::default(),
        };

        let r = batch_handler(user_fn, lambda_event).await.unwrap();
        assert_eq!(r.batch_item_failures.len(), 1);
        assert_eq!(r.batch_item_failures[0].item_identifier, "1");
    }
}
