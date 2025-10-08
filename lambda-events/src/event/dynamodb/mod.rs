use crate::{
    custom_serde::{deserialize_lambda_dynamodb_item, float_unix_epoch},
    streams::DynamoDbBatchItemFailure,
    time_window::*,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;
use std::fmt;

#[cfg(test)]
mod attributes;

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamViewType {
    NewImage,
    OldImage,
    NewAndOldImages,
    #[default]
    KeysOnly,
}

impl fmt::Display for StreamViewType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            StreamViewType::NewImage => "NEW_IMAGE",
            StreamViewType::OldImage => "OLD_IMAGE",
            StreamViewType::NewAndOldImages => "NEW_AND_OLD_IMAGES",
            StreamViewType::KeysOnly => "KEYS_ONLY",
        };
        write!(f, "{val}")
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StreamStatus {
    Enabling,
    Enabled,
    Disabling,
    #[default]
    Disabled,
}

impl fmt::Display for StreamStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            StreamStatus::Enabling => "ENABLING",
            StreamStatus::Enabled => "ENABLED",
            StreamStatus::Disabling => "DISABLING",
            StreamStatus::Disabled => "DISABLED",
        };
        write!(f, "{val}")
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SharedIteratorType {
    TrimHorizon,
    #[default]
    Latest,
    AtSequenceNumber,
    AfterSequenceNumber,
}

impl fmt::Display for SharedIteratorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            SharedIteratorType::TrimHorizon => "TRIM_HORIZON",
            SharedIteratorType::Latest => "LATEST",
            SharedIteratorType::AtSequenceNumber => "AT_SEQUENCE_NUMBER",
            SharedIteratorType::AfterSequenceNumber => "AFTER_SEQUENCE_NUMBER",
        };
        write!(f, "{val}")
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationType {
    #[default]
    Insert,
    Modify,
    Remove,
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            OperationType::Insert => "INSERT",
            OperationType::Modify => "MODIFY",
            OperationType::Remove => "REMOVE",
        };
        write!(f, "{val}")
    }
}

#[non_exhaustive]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KeyType {
    #[default]
    Hash,
    Range,
}

impl fmt::Display for KeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            KeyType::Hash => "HASH",
            KeyType::Range => "RANGE",
        };
        write!(f, "{val}")
    }
}

/// The `Event` stream event handled to Lambda
/// <http://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-ddb-update>
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct Event {
    #[serde(rename = "Records")]
    pub records: Vec<EventRecord>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `TimeWindowEvent` represents an Amazon Dynamodb event when using time windows
/// ref. <https://docs.aws.amazon.com/lambda/latest/dg/with-ddb.html#services-ddb-windows>
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeWindowEvent {
    #[serde(rename = "DynamoDBEvent")]
    #[serde(flatten)]
    pub dynamo_db_event: Event,
    #[serde(rename = "TimeWindowProperties")]
    #[serde(flatten)]
    pub time_window_properties: TimeWindowProperties,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `TimeWindowEventResponse` is the outer structure to report batch item failures for DynamoDBTimeWindowEvent.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeWindowEventResponse {
    #[serde(rename = "TimeWindowEventResponseProperties")]
    #[serde(flatten)]
    pub time_window_event_response_properties: TimeWindowEventResponseProperties,
    pub batch_item_failures: Vec<DynamoDbBatchItemFailure>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// EventRecord stores information about each record of a DynamoDb stream event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRecord {
    /// The region in which the GetRecords request was received.
    pub aws_region: String,
    /// The main body of the stream record, containing all of the DynamoDB-specific
    /// fields.
    #[serde(rename = "dynamodb")]
    pub change: StreamRecord,
    /// A globally unique identifier for the event that was recorded in this stream
    /// record.
    #[serde(rename = "eventID")]
    pub event_id: String,
    /// The type of data modification that was performed on the DynamoDB table:
    ///
    /// * INSERT - a new item was added to the table.
    ///
    /// * MODIFY - one or more of an existing item's attributes were modified.
    ///
    /// * REMOVE - the item was deleted from the table
    pub event_name: String,
    /// The AWS service from which the stream record originated. For DynamoDB Streams,
    /// this is aws:dynamodb.
    #[serde(default)]
    pub event_source: Option<String>,
    /// The version number of the stream record format. This number is updated whenever
    /// the structure of Record is modified.
    ///
    /// Client applications must not assume that eventVersion will remain at a particular
    /// value, as this number is subject to change at any time. In general, eventVersion
    /// will only increase as the low-level DynamoDB Streams API evolves.
    #[serde(default)]
    pub event_version: Option<String>,
    /// The event source ARN of DynamoDB
    #[serde(rename = "eventSourceARN")]
    #[serde(default)]
    pub event_source_arn: Option<String>,
    /// Items that are deleted by the Time to Live process after expiration have
    /// the following fields:
    ///
    /// * Records[].userIdentity.type
    ///
    /// "Service"
    ///
    /// * Records[].userIdentity.principalId
    ///
    /// "dynamodb.amazonaws.com"
    #[serde(default)]
    pub user_identity: Option<UserIdentity>,
    /// Describes the record format and relevant mapping information that
    /// should be applied to schematize the records on the stream. For
    /// DynamoDB Streams, this is application/json.
    #[serde(default)]
    pub record_format: Option<String>,
    /// The DynamoDB table that this event was recorded for.
    #[serde(default)]
    pub table_name: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserIdentity {
    #[serde(default)]
    pub type_: String,
    #[serde(default)]
    pub principal_id: String,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `DynamoDbStreamRecord` represents a description of a single data modification that was performed on an item
/// in a DynamoDB table.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamRecord {
    /// The approximate date and time when the stream record was created, in UNIX
    /// epoch time (<http://www.epochconverter.com/>) format. Might not be present in
    /// the record: <https://github.com/awslabs/aws-lambda-rust-runtime/issues/889>
    #[serde(rename = "ApproximateCreationDateTime")]
    #[serde(with = "float_unix_epoch")]
    #[serde(default)]
    pub approximate_creation_date_time: DateTime<Utc>,
    /// The primary key attribute(s) for the DynamoDB item that was modified.
    #[serde(deserialize_with = "deserialize_lambda_dynamodb_item")]
    #[serde(default)]
    #[serde(rename = "Keys")]
    pub keys: serde_dynamo::Item,
    /// The item in the DynamoDB table as it appeared after it was modified.
    #[serde(deserialize_with = "deserialize_lambda_dynamodb_item")]
    #[serde(default)]
    #[serde(rename = "NewImage")]
    pub new_image: serde_dynamo::Item,
    /// The item in the DynamoDB table as it appeared before it was modified.
    #[serde(deserialize_with = "deserialize_lambda_dynamodb_item")]
    #[serde(default)]
    #[serde(rename = "OldImage")]
    pub old_image: serde_dynamo::Item,
    /// The sequence number of the stream record.
    #[serde(default)]
    #[serde(rename = "SequenceNumber")]
    pub sequence_number: Option<String>,
    /// The size of the stream record, in bytes.
    #[serde(rename = "SizeBytes")]
    pub size_bytes: i64,
    /// The type of data from the modified DynamoDB item that was captured in this
    /// stream record.
    #[serde(default)]
    #[serde(rename = "StreamViewType")]
    pub stream_view_type: Option<StreamViewType>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use super::*;
    use chrono::TimeZone;

    #[test]
    #[cfg(feature = "dynamodb")]
    fn example_dynamodb_event() {
        let data = include_bytes!("../../fixtures/example-dynamodb-event.json");
        let mut parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);

        let event = parsed.records.pop().unwrap();
        let date = Utc.ymd(2016, 12, 2).and_hms(1, 27, 0);
        assert_eq!(date, event.change.approximate_creation_date_time);
    }

    #[test]
    #[cfg(feature = "dynamodb")]
    fn example_dynamodb_event_with_optional_fields() {
        let data = include_bytes!("../../fixtures/example-dynamodb-event-record-with-optional-fields.json");
        let parsed: EventRecord = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: EventRecord = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
        let date = Utc.timestamp_micros(0).unwrap(); // 1970-01-01T00:00:00Z
        assert_eq!(date, reparsed.change.approximate_creation_date_time);
    }
}
