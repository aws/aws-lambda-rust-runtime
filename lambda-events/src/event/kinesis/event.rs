use crate::{
    encodings::{Base64Data, SecondTimestamp},
    time_window::{TimeWindowEventResponseProperties, TimeWindowProperties},
};
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

#[non_exhaustive]
#[derive(Clone, Default, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisEvent {
    #[serde(rename = "Records")]
    pub records: Vec<KinesisEventRecord>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `KinesisTimeWindowEvent` represents an Amazon Dynamodb event when using time windows
/// ref. <https://docs.aws.amazon.com/lambda/latest/dg/with-kinesis.html#services-kinesis-windows>
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisTimeWindowEvent {
    #[serde(rename = "KinesisEvent")]
    #[serde(flatten)]
    pub kinesis_event: KinesisEvent,
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

/// `KinesisTimeWindowEventResponse` is the outer structure to report batch item failures for KinesisTimeWindowEvent.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisTimeWindowEventResponse {
    #[serde(rename = "TimeWindowEventResponseProperties")]
    #[serde(flatten)]
    pub time_window_event_response_properties: TimeWindowEventResponseProperties,
    // pub batch_item_failures: Vec<KinesisBatchItemFailure>,
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
pub struct KinesisEventRecord {
    /// nolint: stylecheck
    #[serde(default)]
    pub aws_region: Option<String>,
    #[serde(default)]
    #[serde(rename = "eventID")]
    pub event_id: Option<String>,
    #[serde(default)]
    pub event_name: Option<String>,
    #[serde(default)]
    pub event_source: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: Option<String>,
    #[serde(default)]
    pub event_version: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub invoke_identity_arn: Option<String>,
    pub kinesis: KinesisRecord,
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
pub struct KinesisRecord {
    pub approximate_arrival_timestamp: SecondTimestamp,
    pub data: Base64Data,
    #[serde(default)]
    pub encryption_type: KinesisEncryptionType,
    #[serde(default)]
    pub partition_key: String,
    #[serde(default)]
    pub sequence_number: String,
    #[serde(default)]
    pub kinesis_schema_version: Option<String>,
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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KinesisEncryptionType {
    #[default]
    None,
    Kms,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "kinesis")]
    fn example_kinesis_event() {
        let data = include_bytes!("../../fixtures/example-kinesis-event.json");
        let parsed: KinesisEvent = serde_json::from_slice(data).unwrap();
        assert_eq!(KinesisEncryptionType::None, parsed.records[0].kinesis.encryption_type);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: KinesisEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "kinesis")]
    fn example_kinesis_event_encrypted() {
        let data = include_bytes!("../../fixtures/example-kinesis-event-encrypted.json");
        let parsed: KinesisEvent = serde_json::from_slice(data).unwrap();
        assert_eq!(KinesisEncryptionType::Kms, parsed.records[0].kinesis.encryption_type);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: KinesisEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    /// `cargo lambda init` autogenerates code that relies on `Default` being implemented for event structs.
    ///
    /// This test validates that `Default` is implemented for each KinesisEvent struct.
    #[test]
    #[cfg(feature = "kinesis")]
    fn test_ensure_default_implemented_for_structs() {
        let _kinesis_event = KinesisEvent::default();
        let _kinesis_time_window_event = KinesisTimeWindowEvent::default();
        let _kinesis_event_record = KinesisEventRecord::default();
        let _kinesis_record = KinesisRecord::default();
        let _kinesis_encryption_type = KinesisEncryptionType::default();
    }
}
