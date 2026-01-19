#[cfg(feature = "builders")]
use bon::Builder;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

/// `KinesisEventResponse` is the outer structure to report batch item failures for KinesisEvent.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisEventResponse {
    pub batch_item_failures: Vec<KinesisBatchItemFailure>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

impl KinesisEventResponse {
    /// Add a failed item identifier to the batch response.
    ///
    /// When processing Kinesis records in batches, you can use this helper method to
    /// register individual record failures. Lambda will automatically retry failed
    /// records while successfully processed records will be checkpointed.
    ///
    /// Besides `item_identifiers`, the generated struct will use default field values for [`KinesisBatchItemFailure`].
    ///
    /// **Important**: This feature requires `FunctionResponseTypes: ReportBatchItemFailures`
    /// to be enabled in your Lambda function's Kinesis event source mapping configuration.
    /// Without this setting, Lambda will retry the entire batch on any failure.
    pub fn add_failure(&mut self, item_identifier: impl Into<String>) {
        self.batch_item_failures.push(KinesisBatchItemFailure {
            item_identifier: Some(item_identifier.into()),
            #[cfg(feature = "catch-all-fields")]
            other: serde_json::Map::new(),
            ..Default::default()
        });
    }

    /// Set multiple failed item identifiers at once.
    ///
    /// This is a convenience method for setting all batch item failures in one call.
    /// It replaces any previously registered failures.
    ///
    /// Besides `item_identifiers`, the generated struct will use default field values for [`KinesisBatchItemFailure`].
    ///
    /// **Important**: This feature requires `FunctionResponseTypes: ReportBatchItemFailures`
    /// to be enabled in your Lambda function's Kinesis event source mapping configuration.
    /// Without this setting, Lambda will retry the entire batch on any failure.
    pub fn set_failures<I, S>(&mut self, item_identifiers: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.batch_item_failures = item_identifiers
            .into_iter()
            .map(|id| KinesisBatchItemFailure {
                item_identifier: Some(id.into()),
                #[cfg(feature = "catch-all-fields")]
                other: serde_json::Map::new(),
                ..Default::default()
            })
            .collect();
    }
}

/// `KinesisBatchItemFailure` is the individual record which failed processing.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisBatchItemFailure {
    #[serde(default)]
    pub item_identifier: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `DynamoDbEventResponse` is the outer structure to report batch item failures for DynamoDBEvent.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamoDbEventResponse {
    pub batch_item_failures: Vec<DynamoDbBatchItemFailure>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `DynamoDbBatchItemFailure` is the individual record which failed processing.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamoDbBatchItemFailure {
    #[serde(default)]
    pub item_identifier: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `SqsEventResponse` is the outer structure to report batch item failures for SQSEvent.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqsEventResponse {
    pub batch_item_failures: Vec<SqsBatchItemFailure>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `SqsBatchItemFailure` is the individual record which failed processing.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqsBatchItemFailure {
    #[serde(default)]
    pub item_identifier: Option<String>,
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
    fn kinesis_event_response_add_failure() {
        let mut response = KinesisEventResponse::default();
        response.add_failure("seq-1");
        response.add_failure("seq-2".to_string());

        assert_eq!(response.batch_item_failures.len(), 2);
        assert_eq!(
            response.batch_item_failures[0].item_identifier,
            Some("seq-1".to_string())
        );
        assert_eq!(
            response.batch_item_failures[1].item_identifier,
            Some("seq-2".to_string())
        );
    }

    #[test]
    fn kinesis_event_response_set_failures() {
        let mut response = KinesisEventResponse::default();
        response.set_failures(vec!["seq-1", "seq-2", "seq-3"]);

        assert_eq!(response.batch_item_failures.len(), 3);
        assert_eq!(
            response.batch_item_failures[0].item_identifier,
            Some("seq-1".to_string())
        );
        assert_eq!(
            response.batch_item_failures[1].item_identifier,
            Some("seq-2".to_string())
        );
        assert_eq!(
            response.batch_item_failures[2].item_identifier,
            Some("seq-3".to_string())
        );

        // Test that set_failures replaces existing failures
        response.set_failures(vec!["seq-4".to_string()]);
        assert_eq!(response.batch_item_failures.len(), 1);
        assert_eq!(
            response.batch_item_failures[0].item_identifier,
            Some("seq-4".to_string())
        );
    }
}
