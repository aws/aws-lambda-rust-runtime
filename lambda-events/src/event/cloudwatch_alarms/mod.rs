use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{
    de::{DeserializeOwned, Visitor},
    ser::Error as SerError,
    Deserialize, Serialize,
};
use serde_json::Value;

/// `CloudWatchAlarm` is the generic outer structure of an event triggered by a CloudWatch Alarm.
/// You probably want to use `CloudWatchMetricAlarm` or `CloudWatchCompositeAlarm` if you know which kind of alarm your function is receiving.
/// For examples of events that come via CloudWatch Alarms,
/// see <https://docs.aws.amazon.com/AmazonCloudWatch/latest/monitoring/AlarmThatSendsEmail.html#Lambda-action-payload>
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchAlarm<C = Value, R = CloudWatchAlarmStateReasonData>
where
    C: DeserializeOwned,
    C: Serialize,
    R: DeserializeOwned,
    R: Serialize,
{
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub alarm_arn: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    pub time: DateTime<Utc>,

    #[serde(default, bound = "")]
    pub alarm_data: CloudWatchAlarmData<C, R>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CloudWatchMetricAlarm` is the structure of an event triggered by CloudWatch metric alarms.
pub type CloudWatchMetricAlarm<R = CloudWatchAlarmStateReasonData> =
    CloudWatchAlarm<CloudWatchMetricAlarmConfiguration, R>;

/// `CloudWatchCompositeAlarm` is the structure of an event triggered by CloudWatch composite alarms.
pub type CloudWatchCompositeAlarm<R = CloudWatchAlarmStateReasonData> =
    CloudWatchAlarm<CloudWatchCompositeAlarmConfiguration, R>;

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchAlarmData<C = Value, R = CloudWatchAlarmStateReasonData>
where
    C: DeserializeOwned,
    C: Serialize,
    R: DeserializeOwned,
    R: Serialize,
{
    pub alarm_name: String,
    #[serde(default, bound = "")]
    pub state: Option<CloudWatchAlarmState<R>>,
    #[serde(default, bound = "")]
    pub previous_state: Option<CloudWatchAlarmState<R>>,
    #[serde(bound = "")]
    pub configuration: C,
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
pub struct CloudWatchAlarmState<R = Value>
where
    R: DeserializeOwned,
    R: Serialize,
{
    #[serde(default)]
    pub value: CloudWatchAlarmStateValue,
    pub reason: String,
    #[serde(default, bound = "")]
    pub reason_data: Option<R>,
    pub timestamp: DateTime<Utc>,
    pub actions_suppressed_by: Option<String>,
    pub actions_suppressed_reason: Option<String>,
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
pub struct CloudWatchMetricAlarmConfiguration {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub metrics: Vec<CloudWatchMetricDefinition>,
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
pub struct CloudWatchMetricDefinition {
    pub id: String,
    #[serde(default)]
    pub return_data: bool,
    pub metric_stat: CloudWatchMetricStatDefinition,
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
pub struct CloudWatchMetricStatDefinition {
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub stat: Option<String>,
    pub period: u16,
    pub metric: CloudWatchMetricStatMetricDefinition,
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
pub struct CloudWatchMetricStatMetricDefinition {
    #[serde(default)]
    pub namespace: Option<String>,
    pub name: String,
    pub dimensions: HashMap<String, String>,
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
pub struct CloudWatchCompositeAlarmConfiguration {
    pub alarm_rule: String,
    pub actions_suppressor: String,
    pub actions_suppressor_wait_period: u16,
    pub actions_suppressor_extension_period: u16,
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
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CloudWatchAlarmStateValue {
    #[default]
    Ok,
    Alarm,
    InsufficientData,
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CloudWatchAlarmStateReasonData {
    Metric(CloudWatchAlarmStateReasonDataMetric),
    Composite(ClodWatchAlarmStateReasonDataComposite),
    Generic(Value),
}

impl Default for CloudWatchAlarmStateReasonData {
    fn default() -> Self {
        Self::Generic(Value::String(String::new()))
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudWatchAlarmStateReasonDataMetric {
    pub version: String,
    #[serde(default)]
    pub query_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub statistic: Option<String>,
    pub period: u16,
    #[serde(default)]
    pub recent_datapoints: Vec<f64>,
    #[serde(default)]
    pub recent_lower_thresholds: Vec<f64>,
    #[serde(default)]
    pub recent_upper_thresholds: Vec<f64>,
    pub threshold: f64,
    #[serde(default)]
    pub evaluated_datapoints: Vec<CloudWatchAlarmStateEvaluatedDatapoint>,
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
#[serde(rename_all = "camelCase")]
pub struct CloudWatchAlarmStateEvaluatedDatapoint {
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub sample_count: Option<f64>,
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub threshold: Option<f64>,
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
pub struct ClodWatchAlarmStateReasonDataComposite {
    #[serde(default)]
    pub triggering_alarms: Vec<CloudWatchAlarmStateTriggeringAlarm>,
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
pub struct CloudWatchAlarmStateTriggeringAlarm {
    pub arn: String,
    pub state: CloudWatchAlarmStateTriggeringAlarmState,
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
pub struct CloudWatchAlarmStateTriggeringAlarmState {
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub value: CloudWatchAlarmStateValue,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

impl<'de> Deserialize<'de> for CloudWatchAlarmStateReasonData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ReasonDataVisitor)
    }
}

impl Serialize for CloudWatchAlarmStateReasonData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let r = match self {
            Self::Metric(m) => serde_json::to_string(m),
            Self::Composite(m) => serde_json::to_string(m),
            Self::Generic(m) => serde_json::to_string(m),
        };
        let s = r.map_err(|e| SerError::custom(format!("failed to serialize struct as string {e}")))?;

        serializer.serialize_str(&s)
    }
}

struct ReasonDataVisitor;

impl Visitor<'_> for ReasonDataVisitor {
    type Value = CloudWatchAlarmStateReasonData;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a string with the alarm state reason data")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Ok(metric) = serde_json::from_str::<CloudWatchAlarmStateReasonDataMetric>(v) {
            return Ok(CloudWatchAlarmStateReasonData::Metric(metric));
        }
        if let Ok(aggregate) = serde_json::from_str::<ClodWatchAlarmStateReasonDataComposite>(v) {
            return Ok(CloudWatchAlarmStateReasonData::Composite(aggregate));
        }
        Ok(CloudWatchAlarmStateReasonData::Generic(Value::String(v.to_owned())))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "cloudwatch_alarms")]
    fn example_cloudwatch_alarm_metric() {
        let data = include_bytes!("../../fixtures/example-cloudwatch-alarm-metric.json");
        let parsed: CloudWatchMetricAlarm = serde_json::from_slice(data).unwrap();
        let state = parsed.alarm_data.previous_state.clone().unwrap();
        let data = state.reason_data.unwrap();
        match &data {
            CloudWatchAlarmStateReasonData::Metric(d) => {
                assert_eq!("1.0", d.version);
                assert_eq!(5, d.evaluated_datapoints.len());
            }
            _ => panic!("unexpected reason data {data:?}"),
        }

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CloudWatchMetricAlarm = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cloudwatch_alarms")]
    fn example_cloudwatch_alarm_composite() {
        let data = include_bytes!("../../fixtures/example-cloudwatch-alarm-composite.json");
        let parsed: CloudWatchCompositeAlarm = serde_json::from_slice(data).unwrap();

        let state = parsed.alarm_data.state.clone().unwrap();
        let data = state.reason_data.unwrap();
        match &data {
            CloudWatchAlarmStateReasonData::Composite(d) => {
                assert_eq!(1, d.triggering_alarms.len());
                assert_eq!(
                    CloudWatchAlarmStateValue::Alarm,
                    d.triggering_alarms.first().unwrap().state.value
                );
            }
            _ => panic!("unexpected reason data {data:?}"),
        }

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CloudWatchCompositeAlarm = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cloudwatch_alarms")]
    fn example_cloudwatch_alarm_composite_with_suppressor_alarm() {
        let data = include_bytes!("../../fixtures/example-cloudwatch-alarm-composite-with-suppressor-alarm.json");
        let parsed: CloudWatchCompositeAlarm = serde_json::from_slice(data).unwrap();
        let state = parsed.alarm_data.state.clone().unwrap();
        assert_eq!("WaitPeriod", state.actions_suppressed_by.unwrap());
        assert_eq!(
            "Actions suppressed by WaitPeriod",
            state.actions_suppressed_reason.unwrap()
        );

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CloudWatchCompositeAlarm = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
