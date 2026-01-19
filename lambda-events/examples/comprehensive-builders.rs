// Example demonstrating builder pattern usage for AWS Lambda events
#[cfg(feature = "builders")]
use aws_lambda_events::event::{
    dynamodb::Event as DynamoDbEvent, kinesis::KinesisEvent, s3::S3Event,
    secretsmanager::SecretsManagerSecretRotationEvent, sns::SnsEvent, sqs::SqsEvent,
};

#[cfg(feature = "builders")]
fn main() {
    // S3 Event - Object storage notifications
    let s3_event = S3Event::builder().records(vec![]).build();

    // Kinesis Event - Stream processing
    let kinesis_event = KinesisEvent::builder().records(vec![]).build();

    // DynamoDB Event - Database change streams
    let dynamodb_event = DynamoDbEvent::builder().records(vec![]).build();

    // SNS Event - Pub/sub messaging
    let sns_event = SnsEvent::builder().records(vec![]).build();

    // SQS Event - Queue messaging
    let sqs_event = SqsEvent::builder().records(vec![]).build();

    // Secrets Manager Event - Secret rotation
    let secrets_event = SecretsManagerSecretRotationEvent::builder()
        .step("createSecret".to_string())
        .secret_id("test-secret".to_string())
        .client_request_token("token-123".to_string())
        .build();
        .unwrap();
}

#[cfg(not(feature = "builders"))]
fn main() {
    println!("This example requires the 'builders' feature to be enabled.");
    println!("Run with: cargo run --example comprehensive-builders --all-features");
}
