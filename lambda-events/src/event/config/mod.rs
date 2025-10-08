use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

/// `ConfigEvent` contains data from an event sent from AWS Config
#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigEvent {
    /// The ID of the AWS account that owns the rule
    #[serde(default)]
    pub account_id: Option<String>,
    /// The ARN that AWS Config assigned to the rule
    ///
    /// nolint:stylecheck
    #[serde(default)]
    pub config_rule_arn: Option<String>,
    /// nolint:stylecheck
    #[serde(default)]
    pub config_rule_id: Option<String>,
    /// The name that you assigned to the rule that caused AWS Config to publish the event
    #[serde(default)]
    pub config_rule_name: Option<String>,
    /// A boolean value that indicates whether the AWS resource to be evaluated has been removed from the rule's scope
    pub event_left_scope: bool,
    /// nolint:stylecheck
    #[serde(default)]
    pub execution_role_arn: Option<String>,
    /// If the event is published in response to a resource configuration change, this value contains a JSON configuration item
    #[serde(default)]
    pub invoking_event: Option<String>,
    /// A token that the function must pass to AWS Config with the PutEvaluations call
    #[serde(default)]
    pub result_token: Option<String>,
    /// Key/value pairs that the function processes as part of its evaluation logic
    #[serde(default)]
    pub rule_parameters: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
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
    #[cfg(feature = "config")]
    fn example_config_event() {
        let data = include_bytes!("../../fixtures/example-config-event.json");
        let parsed: ConfigEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ConfigEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
