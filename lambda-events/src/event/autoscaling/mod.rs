#[cfg(feature = "builders")]
use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::custom_serde::deserialize_lambda_map;

/// `AutoScalingEvent` struct is used to parse the json for auto scaling event types //
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoScalingEvent<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    /// The version of event data
    #[serde(default)]
    pub version: Option<String>,
    /// The unique ID of the event
    #[serde(default)]
    pub id: Option<String>,
    /// Details about event type
    #[serde(default)]
    #[serde(rename = "detail-type")]
    pub detail_type: Option<String>,
    /// Source of the event
    #[serde(default)]
    pub source: Option<String>,
    /// AccountId
    #[serde(default)]
    #[serde(rename = "account")]
    pub account_id: Option<String>,
    /// Event timestamp
    pub time: DateTime<Utc>,
    /// Region of event
    #[serde(default)]
    pub region: Option<String>,
    /// Information about resources impacted by event
    pub resources: Vec<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub detail: HashMap<String, T1>,
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
    #[cfg(feature = "autoscaling")]
    fn example_autoscaling_event_launch_successful() {
        let data = include_bytes!("../../fixtures/example-autoscaling-event-launch-successful.json");
        let parsed: AutoScalingEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AutoScalingEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "autoscaling")]
    fn example_autoscaling_event_launch_unsuccessful() {
        let data = include_bytes!("../../fixtures/example-autoscaling-event-launch-unsuccessful.json");
        let parsed: AutoScalingEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AutoScalingEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "autoscaling")]
    fn example_autoscaling_event_lifecycle_action() {
        let data = include_bytes!("../../fixtures/example-autoscaling-event-lifecycle-action.json");
        let parsed: AutoScalingEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AutoScalingEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "autoscaling")]
    fn example_autoscaling_event_terminate_action() {
        let data = include_bytes!("../../fixtures/example-autoscaling-event-terminate-action.json");
        let parsed: AutoScalingEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AutoScalingEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "autoscaling")]
    fn example_autoscaling_event_terminate_successful() {
        let data = include_bytes!("../../fixtures/example-autoscaling-event-terminate-successful.json");
        let parsed: AutoScalingEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AutoScalingEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "autoscaling")]
    fn example_autoscaling_event_terminate_unsuccessful() {
        let data = include_bytes!("../../fixtures/example-autoscaling-event-terminate-unsuccessful.json");
        let parsed: AutoScalingEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AutoScalingEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
