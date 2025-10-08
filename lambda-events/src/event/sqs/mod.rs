use crate::{custom_serde::deserialize_lambda_map, encodings::Base64Data};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;
use std::collections::HashMap;

/// The Event sent to Lambda from SQS. Contains 1 or more individual SQS Messages
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqsEvent {
    #[serde(rename = "Records")]
    pub records: Vec<SqsMessage>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// An individual SQS Message, its metadata, and Message Attributes
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqsMessage {
    /// nolint: stylecheck
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub receipt_handle: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub md5_of_body: Option<String>,
    #[serde(default)]
    pub md5_of_message_attributes: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub message_attributes: HashMap<String, SqsMessageAttribute>,
    #[serde(default)]
    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: Option<String>,
    #[serde(default)]
    pub event_source: Option<String>,
    #[serde(default)]
    pub aws_region: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// Alternative to `SqsEvent` to be used alongside `SqsMessageObj<T>` when you need to deserialize a nested object into a struct of type `T` within the SQS Message rather than just using the raw SQS Message string
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct SqsEventObj<T: Serialize> {
    #[serde(rename = "Records")]
    #[serde(bound(deserialize = "T: DeserializeOwned"))]
    pub records: Vec<SqsMessageObj<T>>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// Alternative to `SqsMessage` to be used alongside `SqsEventObj<T>` when you need to deserialize a nested object into a struct of type `T` within the SQS Message rather than just using the raw SQS Message string
#[non_exhaustive]
#[serde_with::serde_as]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
#[serde(rename_all = "camelCase")]
pub struct SqsMessageObj<T: Serialize> {
    /// nolint: stylecheck
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub receipt_handle: Option<String>,

    /// Deserialized into a `T` from nested JSON inside the SQS body string. `T` must implement the `Deserialize` or `DeserializeOwned` trait.
    #[serde_as(as = "serde_with::json::JsonString")]
    #[serde(bound(deserialize = "T: DeserializeOwned"))]
    pub body: T,
    #[serde(default)]
    pub md5_of_body: Option<String>,
    #[serde(default)]
    pub md5_of_message_attributes: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub message_attributes: HashMap<String, SqsMessageAttribute>,
    #[serde(default)]
    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: Option<String>,
    #[serde(default)]
    pub event_source: Option<String>,
    #[serde(default)]
    pub aws_region: Option<String>,
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
pub struct SqsMessageAttribute {
    pub string_value: Option<String>,
    pub binary_value: Option<Base64Data>,
    #[serde(default)]
    pub string_list_values: Vec<String>,
    #[serde(default)]
    pub binary_list_values: Vec<Base64Data>,
    #[serde(default)]
    pub data_type: Option<String>,
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
pub struct SqsBatchResponse {
    pub batch_item_failures: Vec<BatchItemFailure>,
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
pub struct BatchItemFailure {
    pub item_identifier: String,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// The Event sent to Lambda from the SQS API. Contains 1 or more individual SQS Messages
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct SqsApiEventObj<T: Serialize> {
    #[serde(bound(deserialize = "T: DeserializeOwned"))]
    pub messages: Vec<SqsApiMessageObj<T>>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// The Event sent to Lambda from SQS API. Contains 1 or more individual SQS Messages
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqsApiEvent {
    pub messages: Vec<SqsApiMessage>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// Alternative to SqsApiEvent to be used alongside `SqsApiMessageObj<T>` when you need to
/// deserialize a nested object into a struct of type T within the SQS Message rather
/// than just using the raw SQS Message string
#[non_exhaustive]
#[serde_with::serde_as]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
#[serde(rename_all = "PascalCase")]
pub struct SqsApiMessageObj<T: Serialize> {
    /// nolint: stylecheck
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub receipt_handle: Option<String>,
    /// Deserialized into a `T` from nested JSON inside the SQS body string. `T` must implement the `Deserialize` or `DeserializeOwned` trait.
    #[serde_as(as = "serde_with::json::JsonString")]
    #[serde(bound(deserialize = "T: DeserializeOwned"))]
    pub body: T,
    #[serde(default)]
    pub md5_of_body: Option<String>,
    #[serde(default)]
    pub md5_of_message_attributes: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub message_attributes: HashMap<String, SqsMessageAttribute>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// An individual SQS API Message, its metadata, and Message Attributes
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SqsApiMessage {
    /// nolint: stylecheck
    #[serde(default)]
    pub message_id: Option<String>,
    #[serde(default)]
    pub receipt_handle: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub md5_of_body: Option<String>,
    #[serde(default)]
    pub md5_of_message_attributes: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub message_attributes: HashMap<String, SqsMessageAttribute>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "sqs")]
    fn example_sqs_event() {
        let data = include_bytes!("../../fixtures/example-sqs-event.json");
        let parsed: SqsEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SqsEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "sqs")]
    fn example_sqs_obj_event() {
        #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
        struct CustStruct {
            a: String,
            b: u32,
        }

        let data = include_bytes!("../../fixtures/example-sqs-event-obj.json");
        let parsed: SqsEventObj<CustStruct> = serde_json::from_slice(data).unwrap();

        assert_eq!(parsed.records[0].body.a, "Test");
        assert_eq!(parsed.records[0].body.b, 123);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SqsEventObj<CustStruct> = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "sqs")]
    fn example_sqs_batch_response() {
        // Example sqs batch response fetched 2022-05-13, from:
        // https://docs.aws.amazon.com/lambda/latest/dg/with-sqs.html#services-sqs-batchfailurereporting
        let data = include_bytes!("../../fixtures/example-sqs-batch-response.json");
        let parsed: SqsBatchResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SqsBatchResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "sqs")]
    fn example_sqs_api_obj_event() {
        // Example sqs api receive message response, fetched 2023-10-23, inspired from:
        // https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_ReceiveMessage.html#API_ReceiveMessage_ResponseSyntax
        #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
        struct CustStruct {
            city: String,
            country: String,
        }

        let data = include_bytes!("../../fixtures/example-sqs-api-event-obj.json");
        let parsed: SqsApiEventObj<CustStruct> = serde_json::from_slice(data).unwrap();

        assert_eq!(parsed.messages[0].body.city, "provincetown");
        assert_eq!(parsed.messages[0].body.country, "usa");

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SqsApiEventObj<CustStruct> = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
