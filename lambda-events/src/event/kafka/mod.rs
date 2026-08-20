use crate::{custom_serde::deserialize_nullish, encodings::MillisecondTimestamp};
#[cfg(feature = "builders")]
use bon::Builder;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;
use std::collections::HashMap;

#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaEvent {
    #[serde(default)]
    pub event_source: Option<String>,
    #[serde(default)]
    pub event_source_arn: Option<String>,
    #[serde(deserialize_with = "deserialize_nullish")]
    #[serde(default)]
    pub records: HashMap<String, Vec<KafkaRecord>>,
    #[serde(default)]
    pub bootstrap_servers: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaRecord {
    #[serde(default)]
    pub topic: Option<String>,
    pub partition: i64,
    pub offset: i64,
    pub timestamp: MillisecondTimestamp,
    #[serde(default)]
    pub timestamp_type: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
    pub headers: Vec<HashMap<String, Vec<i8>>>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `KafkaEventResponse` is the outer structure to report batch item failures for `KafkaEvent`.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaEventResponse {
    pub batch_item_failures: Vec<KafkaBatchItemFailure>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

impl KafkaEventResponse {
    /// Add a failed Kafka item identifier to the batch response.
    ///
    /// Lambda retries the identified records when `ReportBatchItemFailures` is enabled on the
    /// Kafka event source mapping. Returning an error from the handler still retries the whole
    /// batch.
    pub fn add_failure(&mut self, item_identifier: KafkaItemIdentifier) {
        self.batch_item_failures.push(KafkaBatchItemFailure {
            item_identifier,
            ..Default::default()
        });
    }

    /// Set all failed Kafka item identifiers in the batch response.
    ///
    /// This replaces any previously registered failures.
    pub fn set_failures<I>(&mut self, item_identifiers: I)
    where
        I: IntoIterator<Item = KafkaItemIdentifier>,
    {
        self.batch_item_failures = item_identifiers
            .into_iter()
            .map(|item_identifier| KafkaBatchItemFailure {
                item_identifier,
                ..Default::default()
            })
            .collect();
    }
}

/// `KafkaBatchItemFailure` is an individual Kafka record which failed processing.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaBatchItemFailure {
    pub item_identifier: KafkaItemIdentifier,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `KafkaItemIdentifier` identifies a Kafka record for a partial batch response.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KafkaItemIdentifier {
    /// The topic-partition key from the Kafka event's `records` map.
    pub partition: String,
    /// The Kafka record offset.
    pub offset: i64,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "kafka")]
    fn example_kafka_event() {
        let data = include_bytes!("../../fixtures/example-kafka-event.json");
        let parsed: KafkaEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: KafkaEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "kafka")]
    fn kafka_event_response_serializes_item_identifiers() {
        let mut response = KafkaEventResponse::default();
        response.add_failure(KafkaItemIdentifier {
            partition: String::from("some.topic-3"),
            offset: 42,
            ..Default::default()
        });

        let serialized = serde_json::to_value(response).unwrap();

        assert_eq!(
            serialized,
            serde_json::json!({
                "batchItemFailures": [{
                    "itemIdentifier": {
                        "partition": "some.topic-3",
                        "offset": 42,
                    }
                }]
            })
        );
    }

    #[test]
    #[cfg(feature = "kafka")]
    fn kafka_event_response_sets_failures() {
        let mut response = KafkaEventResponse::default();
        response.set_failures([
            KafkaItemIdentifier {
                partition: String::from("some.topic-3"),
                offset: 42,
                ..Default::default()
            },
            KafkaItemIdentifier {
                partition: String::from("some.topic-4"),
                offset: 43,
                ..Default::default()
            },
        ]);

        assert_eq!(response.batch_item_failures.len(), 2);
        assert_eq!(
            response.batch_item_failures[0].item_identifier.partition,
            "some.topic-3"
        );
        assert_eq!(response.batch_item_failures[1].item_identifier.offset, 43);
    }
}
