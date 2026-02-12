#[cfg(feature = "builders")]
use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

/// Parse EventBridge events.
/// Deserialize the event detail into a structure that's `DeserializeOwned`.
///
/// See <https://docs.aws.amazon.com/eventbridge/latest/userguide/eb-events-structure.html> for structure details.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(bound(deserialize = "T1: DeserializeOwned"))]
#[serde(rename_all = "kebab-case")]
pub struct EventBridgeEvent<T1 = Value>
where
    T1: Serialize,
    T1: DeserializeOwned,
{
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    pub detail_type: String,
    pub source: String,
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub resources: Option<Vec<String>>,
    #[serde(bound = "")]
    pub detail: T1,
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
#[cfg(feature = "eventbridge")]
mod test {
    use super::*;

    #[test]
    fn example_eventbridge_obj_event() {
        #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
        #[serde(rename_all = "kebab-case")]
        struct Ec2StateChange {
            instance_id: String,
            state: String,
        }

        // Example from https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/monitoring-instance-state-changes.html
        let data = include_bytes!("../../fixtures/example-eventbridge-event-obj.json");
        let parsed: EventBridgeEvent<Ec2StateChange> = serde_json::from_slice(data).unwrap();

        assert_eq!("i-abcd1111", parsed.detail.instance_id);
        assert_eq!("pending", parsed.detail.state);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: EventBridgeEvent<Ec2StateChange> = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_eventbridge_schedule_event() {
        let data = include_bytes!("../../fixtures/example-eventbridge-schedule.json");
        let parsed: EventBridgeEvent = serde_json::from_slice(data).unwrap();

        assert_eq!("aws.events", parsed.source);
        assert_eq!("Scheduled Event", parsed.detail_type);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: EventBridgeEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
