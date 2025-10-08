use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;
use std::collections::HashMap;

use crate::custom_serde::{deserialize_lambda_map, deserialize_nullish_boolean};

/// The `Event` notification event handled by Lambda
///
/// [https://docs.aws.amazon.com/lambda/latest/dg/with-sns.html](https://docs.aws.amazon.com/lambda/latest/dg/with-sns.html)
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SnsEvent {
    pub records: Vec<SnsRecord>,

    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// SnsRecord stores information about each record of a SNS event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SnsRecord {
    /// A string containing the event source.
    pub event_source: String,

    /// A string containing the event version.
    pub event_version: String,

    /// A string containing the event subscription ARN.
    pub event_subscription_arn: String,

    /// An SNS object representing the SNS message.
    pub sns: SnsMessage,

    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// SnsMessage stores information about each record of a SNS event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SnsMessage {
    /// The type of SNS message. For a lambda event, this should always be **Notification**
    #[serde(rename = "Type")]
    pub sns_message_type: String,

    /// A Universally Unique Identifier, unique for each message published. For a notification that Amazon SNS resends during a retry, the message ID of the original message is used.
    pub message_id: String,

    /// The Amazon Resource Name (ARN) for the topic that this message was published to.
    pub topic_arn: String,

    /// The Subject parameter specified when the notification was published to the topic.
    ///
    /// The SNS Developer Guide states: *This is an optional parameter. If no Subject was specified, then this name-value pair does not appear in this JSON document.*
    ///
    /// Preliminary tests show this appears in the lambda event JSON as `Subject: null`, marking as Option with need to test additional scenarios
    #[serde(default)]
    pub subject: Option<String>,

    /// The time (UTC) when the notification was published.
    pub timestamp: DateTime<Utc>,

    /// Version of the Amazon SNS signature used.
    pub signature_version: String,

    /// Base64-encoded SHA1withRSA signature of the Message, MessageId, Subject (if present), Type, Timestamp, and TopicArn values.
    pub signature: String,

    /// The URL to the certificate that was used to sign the message.
    #[serde(alias = "SigningCertURL")]
    pub signing_cert_url: String,

    /// A URL that you can use to unsubscribe the endpoint from this topic. If you visit this URL, Amazon SNS unsubscribes the endpoint and stops sending notifications to this endpoint.
    #[serde(alias = "UnsubscribeURL")]
    pub unsubscribe_url: String,

    /// The Message value specified when the notification was published to the topic.
    pub message: String,

    /// This is a HashMap of defined attributes for a message. Additional details can be found in the [SNS Developer Guide](https://docs.aws.amazon.com/sns/latest/dg/sns-message-attributes.html)
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub message_attributes: HashMap<String, MessageAttribute>,

    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// An alternate `Event` notification event to use alongside `SnsRecordObj<T>` and `SnsMessageObj<T>` if you want to deserialize an object inside your SNS messages rather than getting an `Option<String>` message
///
/// [https://docs.aws.amazon.com/lambda/latest/dg/with-sns.html](https://docs.aws.amazon.com/lambda/latest/dg/with-sns.html)
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct SnsEventObj<T: Serialize> {
    pub records: Vec<SnsRecordObj<T>>,

    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// Alternative to `SnsRecord`, used alongside `SnsEventObj<T>` and `SnsMessageObj<T>` when deserializing nested objects from within SNS messages)
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct SnsRecordObj<T: Serialize> {
    /// A string containing the event source.
    pub event_source: String,

    /// A string containing the event version.
    pub event_version: String,

    /// A string containing the event subscription ARN.
    pub event_subscription_arn: String,

    /// An SNS object representing the SNS message.
    pub sns: SnsMessageObj<T>,

    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// Alternate version of `SnsMessage` to use in conjunction with `SnsEventObj<T>` and `SnsRecordObj<T>` for deserializing the message into a struct of type `T`
#[non_exhaustive]
#[serde_with::serde_as]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
#[serde(bound(deserialize = "T: DeserializeOwned"))]
pub struct SnsMessageObj<T: Serialize> {
    /// The type of SNS message. For a lambda event, this should always be **Notification**
    #[serde(rename = "Type")]
    pub sns_message_type: String,

    /// A Universally Unique Identifier, unique for each message published. For a notification that Amazon SNS resends during a retry, the message ID of the original message is used.
    pub message_id: String,

    /// The Amazon Resource Name (ARN) for the topic that this message was published to.
    pub topic_arn: String,

    /// The Subject parameter specified when the notification was published to the topic.
    ///
    /// The SNS Developer Guide states: *This is an optional parameter. If no Subject was specified, then this name-value pair does not appear in this JSON document.*
    ///
    /// Preliminary tests show this appears in the lambda event JSON as `Subject: null`, marking as Option with need to test additional scenarios
    #[serde(default)]
    pub subject: Option<String>,

    /// The time (UTC) when the notification was published.
    pub timestamp: DateTime<Utc>,

    /// Version of the Amazon SNS signature used.
    pub signature_version: String,

    /// Base64-encoded SHA1withRSA signature of the Message, MessageId, Subject (if present), Type, Timestamp, and TopicArn values.
    pub signature: String,

    /// The URL to the certificate that was used to sign the message.
    #[serde(alias = "SigningCertURL")]
    pub signing_cert_url: String,

    /// A URL that you can use to unsubscribe the endpoint from this topic. If you visit this URL, Amazon SNS unsubscribes the endpoint and stops sending notifications to this endpoint.
    #[serde(alias = "UnsubscribeURL")]
    pub unsubscribe_url: String,

    /// Deserialized into a `T` from nested JSON inside the SNS message string. `T` must implement the `Deserialize` or `DeserializeOwned` trait.
    #[serde_as(as = "serde_with::json::JsonString")]
    #[serde(bound(deserialize = "T: DeserializeOwned"))]
    pub message: T,

    /// This is a HashMap of defined attributes for a message. Additional details can be found in the [SNS Developer Guide](https://docs.aws.amazon.com/sns/latest/dg/sns-message-attributes.html)
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub message_attributes: HashMap<String, MessageAttribute>,

    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// Structured metadata items (such as timestamps, geospatial data, signatures, and identifiers) about the message.
///
/// Message attributes are optional and separate from—but are sent together with—the message body. The receiver can use this information to decide how to handle the message without having to process the message body first.
///
/// Additional details can be found in the [SNS Developer Guide](https://docs.aws.amazon.com/sns/latest/dg/sns-message-attributes.html)
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct MessageAttribute {
    /// The data type of the attribute. Per the [SNS Developer Guide](https://docs.aws.amazon.com/sns/latest/dg/sns-message-attributes.html), lambda notifications, this will only be **String** or **Binary**.
    #[serde(rename = "Type")]
    pub data_type: String,

    /// The user-specified message attribute value.
    #[serde(rename = "Value")]
    pub value: String,

    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloudWatchAlarmPayload {
    pub alarm_name: String,
    pub alarm_description: String,
    #[serde(rename = "AWSAccountId")]
    pub aws_account_id: String,
    pub new_state_value: String,
    pub new_state_reason: String,
    pub state_change_time: String,
    pub region: String,
    pub alarm_arn: String,
    pub old_state_value: String,
    pub trigger: CloudWatchAlarmTrigger,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloudWatchAlarmTrigger {
    pub period: i64,
    pub evaluation_periods: i64,
    pub comparison_operator: String,
    pub threshold: f64,
    pub treat_missing_data: String,
    pub evaluate_low_sample_count_percentile: String,
    #[serde(default)]
    pub metrics: Vec<CloudWatchMetricDataQuery>,
    pub metric_name: Option<String>,
    pub namespace: Option<String>,
    pub statistic_type: Option<String>,
    pub statistic: Option<String>,
    pub unit: Option<String>,
    #[serde(default)]
    pub dimensions: Vec<CloudWatchDimension>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloudWatchMetricDataQuery {
    pub id: String,
    pub expression: Option<String>,
    pub label: Option<String>,
    pub metric_stat: Option<CloudWatchMetricStat>,
    pub period: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub return_data: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloudWatchMetricStat {
    pub metric: CloudWatchMetric,
    pub period: i64,
    pub stat: String,
    pub unit: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloudWatchMetric {
    #[serde(default)]
    pub dimensions: Vec<CloudWatchDimension>,
    pub metric_name: Option<String>,
    pub namespace: Option<String>,
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
pub struct CloudWatchDimension {
    pub name: String,
    pub value: String,
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
    #[cfg(feature = "sns")]
    fn my_example_sns_event() {
        let data = include_bytes!("../../fixtures/example-sns-event.json");
        let parsed: SnsEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SnsEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "sns")]
    fn my_example_sns_event_pascal_case() {
        let data = include_bytes!("../../fixtures/example-sns-event-pascal-case.json");
        let parsed: SnsEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SnsEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "sns")]
    fn my_example_sns_event_cloudwatch_single_metric() {
        let data = include_bytes!("../../fixtures/example-cloudwatch-alarm-sns-payload-single-metric.json");
        let parsed: SnsEvent = serde_json::from_slice(data).unwrap();
        assert_eq!(1, parsed.records.len());

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SnsEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);

        let parsed: SnsEventObj<CloudWatchAlarmPayload> =
            serde_json::from_slice(data).expect("failed to parse CloudWatch Alarm payload");

        let record = parsed.records.first().unwrap();
        assert_eq!("EXAMPLE", record.sns.message.alarm_name);
    }

    #[test]
    #[cfg(feature = "sns")]
    fn my_example_sns_event_cloudwatch_multiple_metrics() {
        let data = include_bytes!("../../fixtures/example-cloudwatch-alarm-sns-payload-multiple-metrics.json");
        let parsed: SnsEvent = serde_json::from_slice(data).unwrap();
        assert_eq!(2, parsed.records.len());

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SnsEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "sns")]
    fn my_example_sns_obj_event() {
        let data = include_bytes!("../../fixtures/example-sns-event-obj.json");

        #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
        struct CustStruct {
            foo: String,
            bar: i32,
        }

        let parsed: SnsEventObj<CustStruct> = serde_json::from_slice(data).unwrap();
        println!("{parsed:?}");

        assert_eq!(parsed.records[0].sns.message.foo, "Hello world!");
        assert_eq!(parsed.records[0].sns.message.bar, 123);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: SnsEventObj<CustStruct> = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
