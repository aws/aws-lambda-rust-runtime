// Example demonstrating builder pattern usage for AWS Lambda events
#[cfg(feature = "builders")]
use aws_lambda_events::event::{
    dynamodb::EventBuilder as DynamoDbEventBuilder, kinesis::KinesisEventBuilder, s3::S3EventBuilder,
    secretsmanager::SecretsManagerSecretRotationEventBuilder, sns::SnsEventBuilder, sqs::SqsEventBuilder,
};

#[cfg(feature = "builders")]
fn main() {
    // S3 Event - Object storage notifications
    let s3_event = S3EventBuilder::default().records(vec![]).build().unwrap();

    // Kinesis Event - Stream processing
    let kinesis_event = KinesisEventBuilder::default().records(vec![]).build().unwrap();

    // DynamoDB Event - Database change streams
    let dynamodb_event = DynamoDbEventBuilder::default().records(vec![]).build().unwrap();

    // SNS Event - Pub/sub messaging
    let sns_event = SnsEventBuilder::default().records(vec![]).build().unwrap();

    // SQS Event - Queue messaging
    let sqs_event = SqsEventBuilder::default().records(vec![]).build().unwrap();

    // Secrets Manager Event - Secret rotation
    let secrets_event = SecretsManagerSecretRotationEventBuilder::default()
        .step("createSecret")
        .secret_id("test-secret")
        .client_request_token("token-123")
        .build()
        .unwrap();
}

#[cfg(not(feature = "builders"))]
fn main() {
    println!("This example requires the 'builders' feature to be enabled.");
    println!("Run with: cargo run --example comprehensive-builders --all-features");
}
