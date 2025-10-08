use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EcrScanEvent {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    #[serde(rename = "detail-type")]
    pub detail_type: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    pub resources: Vec<String>,
    #[serde(default)]
    pub account: Option<String>,
    pub detail: EcrScanEventDetailType,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EcrScanEventDetailType {
    #[serde(default)]
    #[serde(rename = "scan-status")]
    pub scan_status: Option<String>,
    #[serde(default)]
    #[serde(rename = "repository-name")]
    pub repository_name: Option<String>,
    #[serde(rename = "finding-severity-counts")]
    pub finding_severity_counts: EcrScanEventFindingSeverityCounts,
    #[serde(default)]
    #[serde(rename = "image-digest")]
    pub image_digest: Option<String>,
    #[serde(rename = "image-tags")]
    pub image_tags: Vec<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EcrScanEventFindingSeverityCounts {
    #[serde(default)]
    #[serde(rename = "CRITICAL")]
    pub critical: Option<i64>,
    #[serde(default)]
    #[serde(rename = "HIGH")]
    pub high: Option<i64>,
    #[serde(default)]
    #[serde(rename = "MEDIUM")]
    pub medium: Option<i64>,
    #[serde(default)]
    #[serde(rename = "LOW")]
    pub low: Option<i64>,
    #[serde(default)]
    #[serde(rename = "INFORMATIONAL")]
    pub informational: Option<i64>,
    #[serde(default)]
    #[serde(rename = "UNDEFINED")]
    pub undefined: Option<i64>,
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
    #[cfg(feature = "ecr_scan")]
    fn example_ecr_image_scan_event() {
        let data = include_bytes!("../../fixtures/example-ecr-image-scan-event.json");
        let parsed: EcrScanEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: EcrScanEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "ecr_scan")]
    fn example_ecr_image_scan_event_with_missing_severities() {
        let data = include_bytes!("../../fixtures/example-ecr-image-scan-event-with-missing-severities.json");
        let parsed: EcrScanEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: EcrScanEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
