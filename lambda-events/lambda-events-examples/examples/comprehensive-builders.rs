// Example demonstrating builder pattern usage for AWS Lambda events
use aws_lambda_events::event::{
    dynamodb::{Event as DynamoDbEvent, EventRecord as DynamoDbEventRecord, StreamRecord},
    kinesis::{KinesisEvent, KinesisEventRecord, KinesisRecord, KinesisEncryptionType},
    s3::{S3Event, S3EventRecord, S3Entity, S3Bucket, S3Object, S3RequestParameters, S3UserIdentity},
    secretsmanager::SecretsManagerSecretRotationEvent,
    sns::{SnsEvent, SnsRecord, SnsMessage},
    sqs::{SqsEvent, SqsMessage},
};
use std::collections::HashMap;

fn main() {
    // S3 Event - Object storage notifications with nested structures
    let s3_record = S3EventRecord::builder()
        .event_time(chrono::Utc::now())
        .principal_id(S3UserIdentity::builder().build())
        .request_parameters(S3RequestParameters::builder().build())
        .response_elements(HashMap::new())
        .s3(S3Entity::builder()
            .bucket(S3Bucket::builder().name("my-bucket".to_string()).build())
            .object(S3Object::builder().key("file.txt".to_string()).size(1024).build())
            .build())
        .build();
    let _s3_event = S3Event::builder().records(vec![s3_record]).build();

    // Kinesis Event - Stream processing with data
    let kinesis_record = KinesisEventRecord::builder()
        .kinesis(KinesisRecord::builder()
            .data(serde_json::from_str("\"SGVsbG8gV29ybGQ=\"").unwrap())
            .partition_key("key-1".to_string())
            .sequence_number("12345".to_string())
            .approximate_arrival_timestamp(serde_json::from_str("1234567890.0").unwrap())
            .encryption_type(KinesisEncryptionType::None)
            .build())
        .build();
    let _kinesis_event = KinesisEvent::builder().records(vec![kinesis_record]).build();

    // DynamoDB Event - Database change streams with item data
    let mut keys = HashMap::new();
    keys.insert("id".to_string(), serde_dynamo::AttributeValue::S("123".to_string()));
    
    let dynamodb_record = DynamoDbEventRecord::builder()
        .aws_region("us-east-1".to_string())
        .change(StreamRecord::builder()
            .approximate_creation_date_time(chrono::Utc::now())
            .keys(keys.into())
            .new_image(HashMap::new().into())
            .old_image(HashMap::new().into())
            .size_bytes(100)
            .build())
        .event_id("event-123".to_string())
        .event_name("INSERT".to_string())
        .build();
    let _dynamodb_event = DynamoDbEvent::builder().records(vec![dynamodb_record]).build();

    // SNS Event - Pub/sub messaging with message details
    let sns_record = SnsRecord::builder()
        .event_source("aws:sns".to_string())
        .event_version("1.0".to_string())
        .event_subscription_arn("arn:aws:sns:us-east-1:123456789012:topic".to_string())
        .sns(SnsMessage::builder()
            .message("Hello from SNS".to_string())
            .sns_message_type("Notification".to_string())
            .message_id("msg-123".to_string())
            .topic_arn("arn:aws:sns:us-east-1:123456789012:topic".to_string())
            .timestamp(chrono::Utc::now())
            .signature_version("1".to_string())
            .signature("sig".to_string())
            .signing_cert_url("https://cert.url".to_string())
            .unsubscribe_url("https://unsub.url".to_string())
            .message_attributes(HashMap::new())
            .build())
        .build();
    let _sns_event = SnsEvent::builder().records(vec![sns_record]).build();

    // SQS Event - Queue messaging with attributes
    let mut attrs = HashMap::new();
    attrs.insert("ApproximateReceiveCount".to_string(), "1".to_string());
    attrs.insert("SentTimestamp".to_string(), "1234567890".to_string());
    
    let sqs_message = SqsMessage::builder()
        .attributes(attrs)
        .message_attributes(HashMap::new())
        .body("message body".to_string())
        .message_id("msg-456".to_string())
        .build();
    
    #[cfg(feature = "catch-all-fields")]
    let _sqs_event = SqsEvent::builder()
        .records(vec![sqs_message])
        .other(serde_json::Map::new())
        .build();

    #[cfg(not(feature = "catch-all-fields"))]
    let _sqs_event = SqsEvent::builder().records(vec![sqs_message]).build();

    // Secrets Manager Event - Secret rotation
    let _secrets_event = SecretsManagerSecretRotationEvent::builder()
        .step("createSecret".to_string())
        .secret_id("test-secret".to_string())
        .client_request_token("token-123".to_string())
        .build();
}
