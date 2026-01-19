// Example demonstrating builder pattern usage for AWS Lambda events
#[cfg(feature = "builders")]
use aws_lambda_events::event::{
    dynamodb::Event as DynamoDbEvent, kinesis::KinesisEvent, s3::S3Event,
    secretsmanager::SecretsManagerSecretRotationEvent, sns::SnsEvent, sqs::SqsEvent,
};

#[cfg(feature = "builders")]
fn main() {
    // S3 Event - Object storage notifications
    let _s3_event = S3Event::builder().records(vec![]).build();

    // Kinesis Event - Stream processing
    let _kinesis_event = KinesisEvent::builder().records(vec![]).build();

    // DynamoDB Event - Database change streams
    let _dynamodb_event = DynamoDbEvent::builder().records(vec![]).build();

    // SNS Event - Pub/sub messaging
    let _sns_event = SnsEvent::builder().records(vec![]).build();

    // SQS Event - Queue messaging
    #[cfg(feature = "catch-all-fields")]
    let _sqs_event = SqsEvent::builder()
        .records(vec![])
        .other(serde_json::Map::new())
        .build();
    
    #[cfg(not(feature = "catch-all-fields"))]
    let _sqs_event = SqsEvent::builder().records(vec![]).build();

    // Secrets Manager Event - Secret rotation
    let _secrets_event = SecretsManagerSecretRotationEvent::builder()
        .step("createSecret".to_string())
        .secret_id("test-secret".to_string())
        .client_request_token("token-123".to_string())
        .build();
}

#[cfg(not(feature = "builders"))]
fn main() {
    println!("This example requires the 'builders' feature to be enabled.");
    println!("Run with: cargo run --example comprehensive-builders --all-features");
}
