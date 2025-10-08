use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

use crate::custom_serde::deserialize_nullish_boolean;

/// `CodeCommitEvent` represents a CodeCommit event
#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeCommitEvent {
    #[serde(rename = "Records")]
    pub records: Vec<CodeCommitRecord>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

pub type CodeCommitEventTime = DateTime<Utc>;

/// `CodeCommitRecord` represents a CodeCommit record
#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeCommitRecord {
    #[serde(default)]
    pub event_id: Option<String>,
    #[serde(default)]
    pub event_version: Option<String>,
    pub event_time: CodeCommitEventTime,
    #[serde(default)]
    pub event_trigger_name: Option<String>,
    pub event_part_number: u64,
    #[serde(rename = "codecommit")]
    pub code_commit: CodeCommitCodeCommit,
    #[serde(default)]
    pub event_name: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub event_trigger_config_id: Option<String>,
    #[serde(default)]
    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: Option<String>,
    #[serde(default)]
    #[serde(rename = "userIdentityARN")]
    pub user_identity_arn: Option<String>,
    #[serde(default)]
    pub event_source: Option<String>,
    #[serde(default)]
    pub aws_region: Option<String>,
    pub event_total_parts: u64,
    pub custom_data: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CodeCommitCodeCommit` represents a CodeCommit object in a record
#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeCommitCodeCommit {
    pub references: Vec<CodeCommitReference>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CodeCommitReference` represents a Reference object in a CodeCommit object
#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeCommitReference {
    #[serde(default)]
    pub commit: Option<String>,
    #[serde(default)]
    pub ref_: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub created: bool,
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
    #[cfg(feature = "code_commit")]
    fn example_code_commit_event() {
        let data = include_bytes!("../../fixtures/example-code_commit-event.json");
        let parsed: CodeCommitEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CodeCommitEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
