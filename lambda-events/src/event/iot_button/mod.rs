#[cfg(feature = "builders")]
use bon::Builder;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTButtonEvent {
    #[serde(default)]
    pub serial_number: Option<String>,
    #[serde(default)]
    pub click_type: Option<String>,
    #[serde(default)]
    pub battery_voltage: Option<String>,
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
    #[cfg(feature = "iot_button")]
    fn example_iot_button_event() {
        let data = include_bytes!("../../fixtures/example-iot_button-event.json");
        let parsed: IoTButtonEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: IoTButtonEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
