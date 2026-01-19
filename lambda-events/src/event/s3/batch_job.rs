#[cfg(feature = "builders")]
use bon::Builder;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

/// `S3BatchJobEvent` encapsulates the detail of a s3 batch job
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BatchJobEvent {
    #[serde(default)]
    pub invocation_schema_version: Option<String>,
    #[serde(default)]
    pub invocation_id: Option<String>,
    pub job: S3BatchJob,
    pub tasks: Vec<S3BatchJobTask>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `S3BatchJob` whichs have the job id
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BatchJob {
    #[serde(default)]
    pub id: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `S3BatchJobTask` represents one task in the s3 batch job and have all task details
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BatchJobTask {
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub s3_key: Option<String>,
    #[serde(default)]
    pub s3_version_id: Option<String>,
    #[serde(default)]
    pub s3_bucket_arn: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `S3BatchJobResponse` is the response of a iven s3 batch job with the results
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BatchJobResponse {
    #[serde(default)]
    pub invocation_schema_version: Option<String>,
    #[serde(default)]
    pub treat_missing_keys_as: Option<String>,
    #[serde(default)]
    pub invocation_id: Option<String>,
    pub results: Vec<S3BatchJobResult>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `S3BatchJobResult` represents the result of a given task
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BatchJobResult {
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub result_code: Option<String>,
    #[serde(default)]
    pub result_string: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}
