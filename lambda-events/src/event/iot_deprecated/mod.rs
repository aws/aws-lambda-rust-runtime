use crate::iot::*;
#[cfg(feature = "builders")]
use bon::Builder;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

/// `IoTCustomAuthorizerRequest` contains data coming in to a custom IoT device gateway authorizer function.
/// Deprecated: Use IoTCoreCustomAuthorizerRequest instead. `IoTCustomAuthorizerRequest` does not correctly model the request schema
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCustomAuthorizerRequest {
    pub http_context: Option<IoTHttpContext>,
    pub mqtt_context: Option<IoTMqttContext>,
    pub tls_context: Option<IoTTlsContext>,
    #[serde(default)]
    #[serde(rename = "token")]
    pub authorization_token: Option<String>,
    #[serde(default)]
    pub token_signature: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

pub type IoTHttpContext = IoTCoreHttpContext;

pub type IoTMqttContext = IoTCoreMqttContext;

pub type IoTTlsContext = IoTCoreTlsContext;

/// `IoTCustomAuthorizerResponse` represents the expected format of an IoT device gateway authorization response.
/// Deprecated: Use IoTCoreCustomAuthorizerResponse. `IoTCustomAuthorizerResponse` does not correctly model the response schema.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCustomAuthorizerResponse {
    pub is_authenticated: bool,
    #[serde(default)]
    pub principal_id: Option<String>,
    pub disconnect_after_in_seconds: i32,
    pub refresh_after_in_seconds: i32,
    pub policy_documents: Vec<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}
