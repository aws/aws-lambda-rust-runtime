use crate::{
    custom_serde::{serialize_headers, serialize_non_null},
    encodings::Base64Data,
    iam::IamPolicyDocument,
};
#[cfg(feature = "builders")]
use bon::Builder;
use http::HeaderMap;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

/// `IoTCoreCustomAuthorizerRequest` represents the request to an IoT Core custom authorizer.
/// See <https://docs.aws.amazon.com/iot/latest/developerguide/config-custom-auth.html>
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCoreCustomAuthorizerRequest {
    #[serde(default)]
    pub token: Option<String>,
    pub signature_verified: bool,
    pub protocols: Vec<String>,
    pub protocol_data: Option<IoTCoreProtocolData>,
    pub connection_metadata: Option<IoTCoreConnectionMetadata>,
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
pub struct IoTCoreProtocolData {
    pub tls: Option<IoTCoreTlsContext>,
    pub http: Option<IoTCoreHttpContext>,
    pub mqtt: Option<IoTCoreMqttContext>,
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
pub struct IoTCoreTlsContext {
    #[serde(default)]
    pub server_name: Option<String>,
    #[serde(default)]
    pub x509_certificate_pem: Option<String>,
    #[serde(default)]
    pub principal_id: Option<String>,
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
pub struct IoTCoreHttpContext {
    #[serde(deserialize_with = "http_serde::header_map::deserialize", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(default)]
    pub query_string: Option<String>,
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
pub struct IoTCoreMqttContext {
    #[serde(default)]
    pub client_id: Option<String>,
    ///  X.509 custom authorizer requests don't include a password field.
    /// Default to empty `Vec<u8>` when absent.
    /// Serializing result will be `password: ""`
    #[serde(default)]
    pub password: Base64Data,
    #[serde(default)]
    pub username: Option<String>,
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
pub struct IoTCoreConnectionMetadata {
    #[serde(default)]
    pub id: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `IoTCoreCustomAuthorizerResponse` represents the response from an IoT Core custom authorizer.
/// See <https://docs.aws.amazon.com/iot/latest/developerguide/config-custom-auth.html>
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IoTCoreCustomAuthorizerResponse {
    pub is_authenticated: bool,
    #[serde(default)]
    pub principal_id: Option<String>,
    pub disconnect_after_in_seconds: u32,
    pub refresh_after_in_seconds: u32,
    /// Policy documents returned to AWS IoT. `None` entries are omitted during serialization because AWS IoT rejects
    /// null policy documents.
    #[serde(serialize_with = "serialize_non_null")]
    pub policy_documents: Vec<Option<IamPolicyDocument>>,
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

    fn custom_auth_response_with_policy_documents(
        policy_documents: Vec<Option<IamPolicyDocument>>,
    ) -> IoTCoreCustomAuthorizerResponse {
        let data = include_bytes!("../../fixtures/example-iot-custom-auth-response.json");
        let mut response: IoTCoreCustomAuthorizerResponse = serde_json::from_slice(data).unwrap();
        response.policy_documents = policy_documents;
        response
    }

    fn policy_document(version: &str) -> IamPolicyDocument {
        serde_json::from_value(serde_json::json!({
            "Version": version,
            "Statement": []
        }))
        .unwrap()
    }

    fn serialized_policy_documents(policy_documents: Vec<Option<IamPolicyDocument>>) -> serde_json::Value {
        let response = custom_auth_response_with_policy_documents(policy_documents);
        serde_json::to_value(response).unwrap()["policyDocuments"].clone()
    }

    #[test]
    #[cfg(feature = "iot")]
    fn example_iot_custom_auth_request() {
        let data = include_bytes!("../../fixtures/example-iot-custom-auth-request.json");
        let parsed: IoTCoreCustomAuthorizerRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: IoTCoreCustomAuthorizerRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "iot")]
    fn example_iot_custom_auth_request_x509() {
        let data = include_bytes!("../../fixtures/example-iot-custom-auth-request-x509.json");
        let parsed: IoTCoreCustomAuthorizerRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: IoTCoreCustomAuthorizerRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "iot")]
    fn example_iot_custom_auth_response() {
        let data = include_bytes!("../../fixtures/example-iot-custom-auth-response.json");
        let parsed: IoTCoreCustomAuthorizerResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: IoTCoreCustomAuthorizerResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn iot_custom_auth_response_serializes_policy_document() {
        let policy = policy_document("2012-10-17");

        assert_eq!(
            serde_json::json!([policy]),
            serialized_policy_documents(vec![Some(policy)])
        );
    }

    #[test]
    fn iot_custom_auth_response_filters_none_policy_documents() {
        let first_policy = policy_document("2012-10-17");
        let second_policy = policy_document("2008-10-17");

        assert_eq!(
            serde_json::json!([first_policy, second_policy]),
            serialized_policy_documents(vec![Some(first_policy), None, Some(second_policy)])
        );
    }

    #[test]
    fn iot_custom_auth_response_serializes_only_none_as_empty_array() {
        assert_eq!(serde_json::json!([]), serialized_policy_documents(vec![None]));
    }

    #[test]
    fn iot_custom_auth_response_serializes_empty_policy_documents_as_empty_array() {
        assert_eq!(serde_json::json!([]), serialized_policy_documents(vec![]));
    }

    #[test]
    fn iot_custom_auth_response_deserializes_null_policy_document() {
        let response = custom_auth_response_with_policy_documents(vec![None]);
        let mut serialized = serde_json::to_value(response).unwrap();
        serialized["policyDocuments"] = serde_json::json!([null]);

        let deserialized: IoTCoreCustomAuthorizerResponse = serde_json::from_value(serialized).unwrap();

        assert_eq!(vec![None], deserialized.policy_documents);
    }
}
