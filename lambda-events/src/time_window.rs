use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::custom_serde::deserialize_lambda_map;

/// `Window` is the object that captures the time window for the records in the event when using the tumbling windows feature
/// Kinesis: <https://docs.aws.amazon.com/lambda/latest/dg/with-kinesis.html#services-kinesis-windows>
/// DDB: <https://docs.aws.amazon.com/lambda/latest/dg/with-ddb.html#services-ddb-windows>
#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl Default for Window {
    fn default() -> Self {
        Window {
            start: Utc::now(),
            end: Utc::now(),
        }
    }
}

/// `TimeWindowProperties` is the object that captures properties that relate to the tumbling windows feature
/// Kinesis: <https://docs.aws.amazon.com/lambda/latest/dg/with-kinesis.html#services-kinesis-windows>
/// DDB: <https://docs.aws.amazon.com/lambda/latest/dg/with-ddb.html#services-ddb-windows>
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeWindowProperties {
    /// Time window for the records in the event.
    pub window: Window,
    /// State being built up to this invoke in the time window.
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub state: HashMap<String, String>,
    /// Shard id of the records
    #[serde(default)]
    pub shard_id: Option<String>,
    /// The event source ARN of the service that generated the event (eg. DynamoDB or Kinesis)
    #[serde(default)]
    #[serde(rename = "eventSourceARN")]
    pub event_source_arn: Option<String>,
    /// Set to true for the last invoke of the time window.
    /// Subsequent invoke will start a new time window along with a fresh state.
    pub is_final_invoke_for_window: bool,
    /// Set to true if window is terminated prematurely.
    /// Subsequent invoke will continue the same window with a fresh state.
    pub is_window_terminated_early: bool,
}

/// `TimeWindowEventResponseProperties` is the object that captures response properties that relate to the tumbling windows feature
/// Kinesis: <https://docs.aws.amazon.com/lambda/latest/dg/with-kinesis.html#services-kinesis-windows>
/// DDB: <https://docs.aws.amazon.com/lambda/latest/dg/with-ddb.html#services-ddb-windows>
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeWindowEventResponseProperties {
    /// State being built up to this invoke in the time window.
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub state: HashMap<String, String>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_window_deserializer() {
        let v = serde_json::json!({
            "start": "2020-12-09T07:04:00Z",
            "end": "2020-12-09T07:06:00Z",
        });

        let parsed: Window = serde_json::from_value(v).unwrap();
        assert_eq!("2020-12-09T07:04:00+00:00", &parsed.start.to_rfc3339());
        assert_eq!("2020-12-09T07:06:00+00:00", &parsed.end.to_rfc3339());
    }
}
