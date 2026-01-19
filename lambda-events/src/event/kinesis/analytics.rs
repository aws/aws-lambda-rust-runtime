use crate::encodings::Base64Data;
#[cfg(feature = "builders")]
use bon::Builder;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisAnalyticsOutputDeliveryEvent {
    #[serde(default)]
    pub invocation_id: Option<String>,
    #[serde(default)]
    pub application_arn: Option<String>,
    pub records: Vec<KinesisAnalyticsOutputDeliveryEventRecord>,
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
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisAnalyticsOutputDeliveryEventRecord {
    #[serde(default)]
    pub record_id: Option<String>,
    pub data: Base64Data,
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
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisAnalyticsOutputDeliveryResponse {
    pub records: Vec<KinesisAnalyticsOutputDeliveryResponseRecord>,
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
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KinesisAnalyticsOutputDeliveryResponseRecord {
    #[serde(default)]
    pub record_id: Option<String>,
    /// possible values include Ok and DeliveryFailed
    #[serde(default)]
    pub result: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}
