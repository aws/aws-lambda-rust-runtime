#[cfg(feature = "builders")]
use bon::Builder;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientVpnConnectionHandlerRequest {
    #[serde(default)]
    #[serde(rename = "connection-id")]
    pub connection_id: Option<String>,
    #[serde(default)]
    #[serde(rename = "endpoint-id")]
    pub endpoint_id: Option<String>,
    #[serde(default)]
    #[serde(rename = "common-name")]
    pub common_name: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    #[serde(rename = "platform")]
    pub os_platform: Option<String>,
    #[serde(default)]
    #[serde(rename = "platform-version")]
    pub os_platform_version: Option<String>,
    #[serde(default)]
    #[serde(rename = "public-ip")]
    pub public_ip: Option<String>,
    #[serde(default)]
    #[serde(rename = "client-openvpn-version")]
    pub client_open_vpn_version: Option<String>,
    #[serde(default)]
    #[serde(rename = "schema-version")]
    pub schema_version: Option<String>,
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
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientVpnConnectionHandlerResponse {
    pub allow: bool,
    #[serde(default)]
    #[serde(rename = "error-msg-on-failed-posture-compliance")]
    pub error_msg_on_failed_posture_compliance: Option<String>,
    #[serde(rename = "posture-compliance-statuses")]
    pub posture_compliance_statuses: Vec<String>,
    #[serde(default)]
    #[serde(rename = "schema-version")]
    pub schema_version: Option<String>,
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
    #[cfg(feature = "clientvpn")]
    fn example_clientvpn_connectionhandler_request() {
        let data = include_bytes!("../../fixtures/example-clientvpn-connectionhandler-request.json");
        let parsed: ClientVpnConnectionHandlerRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ClientVpnConnectionHandlerRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
