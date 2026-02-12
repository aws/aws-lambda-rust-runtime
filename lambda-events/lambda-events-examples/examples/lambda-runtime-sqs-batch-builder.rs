// Example showing how builders simplify SQS batch response construction
// when handling partial batch failures

use aws_lambda_events::event::sqs::{BatchItemFailure, SqsBatchResponse, SqsEvent};
use lambda_runtime::{Error, LambdaEvent};

// Simulate processing a record
#[allow(dead_code)]
async fn process_record(record: &aws_lambda_events::event::sqs::SqsMessage) -> Result<(), String> {
    // Simulate some processing logic
    if let Some(body) = &record.body {
        if body.contains("error") {
            return Err(format!("Failed to process message: {}", body));
        }
    }
    Ok(())
}

// Lambda handler using builder pattern
#[allow(dead_code)]
pub async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<SqsBatchResponse, Error> {
    let mut batch_item_failures = Vec::new();

    for record in event.payload.records {
        match process_record(&record).await {
            Ok(_) => (),
            Err(_) => {
                let item = BatchItemFailure::builder()
                    .item_identifier(record.message_id.unwrap())
                    .build();

                batch_item_failures.push(item)
            }
        }
    }

    let response = SqsBatchResponse::builder()
        .batch_item_failures(batch_item_failures)
        .build();

    Ok(response)
}

fn main() {
    // Demonstrate builder usage with sample data
    let failures = vec![
        BatchItemFailure::builder()
            .item_identifier("msg-123".to_string())
            .build(),
        BatchItemFailure::builder()
            .item_identifier("msg-456".to_string())
            .build(),
    ];

    let response = SqsBatchResponse::builder().batch_item_failures(failures).build();

    println!(
        "Built SQS batch response with {} failed items",
        response.batch_item_failures.len()
    );
    for failure in &response.batch_item_failures {
        println!("   Failed message: {}", failure.item_identifier);
    }
}
