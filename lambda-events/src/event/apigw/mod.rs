use crate::{
    custom_serde::{
        deserialize_headers, deserialize_lambda_map, deserialize_nullish_boolean, http_method, serialize_headers,
        serialize_multi_value_headers,
    },
    encodings::Body,
    iam::IamPolicyStatement,
};
use http::{HeaderMap, Method};
use query_map::QueryMap;
use serde::{de::DeserializeOwned, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;

/// `ApiGatewayProxyRequest` contains data coming from the API Gateway proxy
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayProxyRequest {
    /// The resource path defined in API Gateway
    #[serde(default)]
    pub resource: Option<String>,
    /// The url path for the caller
    #[serde(default)]
    pub path: Option<String>,
    #[serde(with = "http_method")]
    pub http_method: Method,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    #[serde(serialize_with = "query_map::serde::aws_api_gateway_v1::serialize_query_string_parameters")]
    pub query_string_parameters: QueryMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    pub multi_value_query_string_parameters: QueryMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
    #[serde(bound = "")]
    pub request_context: ApiGatewayProxyRequestContext,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayProxyResponse` configures the response to be returned by API Gateway for the request
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayProxyResponse {
    pub status_code: i64,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Body>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayProxyRequestContext` contains the information to identify the AWS account and resources invoking the
/// Lambda function. It also includes Cognito identity information for the caller.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayProxyRequestContext {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_name: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub domain_name: Option<String>,
    #[serde(default)]
    pub domain_prefix: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub identity: ApiGatewayRequestIdentity,
    #[serde(default)]
    pub resource_path: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_authorizer_fields",
        serialize_with = "serialize_authorizer_fields",
        skip_serializing_if = "ApiGatewayRequestAuthorizer::is_empty"
    )]
    pub authorizer: ApiGatewayRequestAuthorizer,
    #[serde(with = "http_method")]
    pub http_method: Method,
    #[serde(default)]
    pub request_time: Option<String>,
    #[serde(default)]
    pub request_time_epoch: i64,
    /// The API Gateway rest API Id
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayV2httpRequest` contains data coming from the new HTTP API Gateway
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequest {
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
    #[serde(default)]
    pub method_arn: Option<String>,
    #[serde(with = "http_method", default = "default_http_method")]
    pub http_method: Method,
    #[serde(default)]
    pub identity_source: Option<String>,
    #[serde(default)]
    pub authorization_token: Option<String>,
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub route_key: Option<String>,
    #[serde(default, alias = "path")]
    pub raw_path: Option<String>,
    #[serde(default)]
    pub raw_query_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookies: Option<Vec<String>>,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(
        default,
        deserialize_with = "query_map::serde::aws_api_gateway_v2::deserialize_empty"
    )]
    #[serde(skip_serializing_if = "QueryMap::is_empty")]
    #[serde(serialize_with = "query_map::serde::aws_api_gateway_v2::serialize_query_string_parameters")]
    pub query_string_parameters: QueryMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub path_parameters: HashMap<String, String>,
    pub request_context: ApiGatewayV2httpRequestContext,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default)]
    pub is_base64_encoded: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayV2httpRequestContext` contains the information to identify the AWS account and resources invoking the Lambda function.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContext {
    #[serde(default)]
    pub route_key: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorizer: Option<ApiGatewayRequestAuthorizer>,
    /// The API Gateway HTTP API Id
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
    #[serde(default)]
    pub domain_name: Option<String>,
    #[serde(default)]
    pub domain_prefix: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub time_epoch: i64,
    pub http: ApiGatewayV2httpRequestContextHttpDescription,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<ApiGatewayV2httpRequestContextAuthentication>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayRequestAuthorizer` contains authorizer information for the request context.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ApiGatewayRequestAuthorizer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwt: Option<ApiGatewayRequestAuthorizerJwtDescription>,
    #[serde(
        bound = "",
        rename = "lambda",
        default,
        skip_serializing_if = "HashMap::is_empty",
        deserialize_with = "deserialize_lambda_map"
    )]
    pub fields: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iam: Option<ApiGatewayRequestAuthorizerIamDescription>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayRequestAuthorizerJwtDescription` contains JWT authorizer information for the request context.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayRequestAuthorizerJwtDescription {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub claims: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayRequestAuthorizerIamDescription` contains IAM information for the request context.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayRequestAuthorizerIamDescription {
    #[serde(default)]
    pub access_key: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub caller_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cognito_identity: Option<ApiGatewayRequestAuthorizerCognitoIdentity>,
    #[serde(default)]
    pub principal_org_id: Option<String>,
    #[serde(default)]
    pub user_arn: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayRequestAuthorizerCognitoIdentity` contains Cognito identity information for the request context.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayRequestAuthorizerCognitoIdentity {
    pub amr: Vec<String>,
    #[serde(default)]
    pub identity_id: Option<String>,
    #[serde(default)]
    pub identity_pool_id: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayV2httpRequestContextHttpDescription` contains HTTP information for the request context.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextHttpDescription {
    #[serde(with = "http_method")]
    pub method: Method,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayV2httpResponse` configures the response to be returned by API Gateway V2 for the request
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpResponse {
    pub status_code: i64,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Body>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,
    pub cookies: Vec<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayRequestIdentity` contains identity information for the request caller.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayRequestIdentity {
    #[serde(default)]
    pub cognito_identity_pool_id: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub cognito_identity_id: Option<String>,
    #[serde(default)]
    pub caller: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub api_key_id: Option<String>,
    #[serde(default)]
    pub access_key: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    #[serde(default)]
    pub cognito_authentication_type: Option<String>,
    #[serde(default)]
    pub cognito_authentication_provider: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub user_arn: Option<String>,
    #[serde(default)]
    pub user_agent: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayWebsocketProxyRequest` contains data coming from the API Gateway proxy
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayWebsocketProxyRequest {
    /// The resource path defined in API Gateway
    #[serde(default)]
    pub resource: Option<String>,
    /// The url path for the caller
    #[serde(default)]
    pub path: Option<String>,
    #[serde(deserialize_with = "http_method::deserialize_optional")]
    #[serde(serialize_with = "http_method::serialize_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub http_method: Option<Method>,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    pub query_string_parameters: QueryMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    pub multi_value_query_string_parameters: QueryMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
    #[serde(bound = "")]
    pub request_context: ApiGatewayWebsocketProxyRequestContext,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayWebsocketProxyRequestContext` contains the information to identify
/// the AWS account and resources invoking the Lambda function. It also includes
/// Cognito identity information for the caller.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayWebsocketProxyRequestContext {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub identity: ApiGatewayRequestIdentity,
    #[serde(default)]
    pub resource_path: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_authorizer_fields",
        serialize_with = "serialize_authorizer_fields",
        skip_serializing_if = "ApiGatewayRequestAuthorizer::is_empty"
    )]
    pub authorizer: ApiGatewayRequestAuthorizer,
    #[serde(deserialize_with = "http_method::deserialize_optional")]
    #[serde(serialize_with = "http_method::serialize_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub http_method: Option<Method>,
    /// The API Gateway rest API Id
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
    pub connected_at: i64,
    #[serde(default)]
    pub connection_id: Option<String>,
    #[serde(default)]
    pub domain_name: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub extended_request_id: Option<String>,
    #[serde(default)]
    pub integration_latency: Option<String>,
    #[serde(default)]
    pub message_direction: Option<String>,
    #[serde(bound = "")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub request_time: Option<String>,
    pub request_time_epoch: i64,
    #[serde(default)]
    pub route_key: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub disconnect_status_code: Option<i64>,
    #[serde(default)]
    pub disconnect_reason: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayCustomAuthorizerRequestTypeRequestIdentity` contains identity information for the request caller including certificate information if using mTLS.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequestTypeRequestIdentity {
    #[serde(default)]
    pub api_key_id: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub source_ip: Option<String>,
    #[serde(default)]
    pub client_cert: Option<ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCert>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCert` contains certificate information for the request caller if using mTLS.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCert {
    #[serde(default)]
    pub client_cert_pem: Option<String>,
    #[serde(default)]
    #[serde(rename = "issuerDN")]
    pub issuer_dn: Option<String>,
    #[serde(default)]
    pub serial_number: Option<String>,
    #[serde(default)]
    #[serde(rename = "subjectDN")]
    pub subject_dn: Option<String>,
    pub validity: ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCertValidity,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCertValidity` contains certificate validity information for the request caller if using mTLS.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequestTypeRequestIdentityClientCertValidity {
    #[serde(default)]
    pub not_after: Option<String>,
    #[serde(default)]
    pub not_before: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayV2httpRequestContextAuthentication` contains authentication context information for the request caller including client certificate information if using mTLS.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextAuthentication {
    #[serde(default)]
    pub client_cert: Option<ApiGatewayV2httpRequestContextAuthenticationClientCert>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayV2httpRequestContextAuthenticationClientCert` contains client certificate information for the request caller if using mTLS.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextAuthenticationClientCert {
    #[serde(default)]
    pub client_cert_pem: Option<String>,
    #[serde(default)]
    #[serde(rename = "issuerDN")]
    pub issuer_dn: Option<String>,
    #[serde(default)]
    pub serial_number: Option<String>,
    #[serde(default)]
    #[serde(rename = "subjectDN")]
    pub subject_dn: Option<String>,
    pub validity: ApiGatewayV2httpRequestContextAuthenticationClientCertValidity,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayV2httpRequestContextAuthenticationClientCertValidity` contains client certificate validity information for the request caller if using mTLS.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2httpRequestContextAuthenticationClientCertValidity {
    #[serde(default)]
    pub not_after: Option<String>,
    #[serde(default)]
    pub not_before: Option<String>,
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
pub struct ApiGatewayV2CustomAuthorizerV1RequestTypeRequestContext {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    pub identity: ApiGatewayCustomAuthorizerRequestTypeRequestIdentity,
    #[serde(default)]
    pub resource_path: Option<String>,
    #[serde(with = "http_method")]
    pub http_method: Method,
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
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
pub struct ApiGatewayV2CustomAuthorizerV1Request {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub type_: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub method_arn: Option<String>,
    #[serde(default)]
    pub identity_source: Option<String>,
    #[serde(default)]
    pub authorization_token: Option<String>,
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(with = "http_method")]
    pub http_method: Method,
    #[serde(deserialize_with = "http_serde::header_map::deserialize", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub query_string_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
    pub request_context: ApiGatewayV2CustomAuthorizerV1RequestTypeRequestContext,
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
pub struct ApiGatewayV2CustomAuthorizerV2Request {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub type_: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub route_arn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_source: Option<Vec<String>>,
    #[serde(default)]
    pub route_key: Option<String>,
    #[serde(default)]
    pub raw_path: Option<String>,
    #[serde(default)]
    pub raw_query_string: Option<String>,
    #[serde(default)]
    pub cookies: Vec<String>,
    #[serde(deserialize_with = "http_serde::header_map::deserialize", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub query_string_parameters: HashMap<String, String>,
    pub request_context: ApiGatewayV2httpRequestContext,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayCustomAuthorizerContext` represents the expected format of an API Gateway custom authorizer response.
/// Deprecated. Code should be updated to use the Authorizer map from APIGatewayRequestIdentity. Ex: Authorizer["principalId"]
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerContext {
    pub principal_id: Option<String>,
    pub string_key: Option<String>,
    pub num_key: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub bool_key: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayCustomAuthorizerRequestTypeRequestContext` represents the expected format of an API Gateway custom authorizer response.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequestTypeRequestContext {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub stage: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub identity: Option<ApiGatewayCustomAuthorizerRequestTypeRequestIdentity>,
    #[serde(default)]
    pub resource_path: Option<String>,
    #[serde(deserialize_with = "http_method::deserialize_optional")]
    #[serde(serialize_with = "http_method::serialize_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub http_method: Option<Method>,
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayCustomAuthorizerRequest` contains data coming in to a custom API Gateway authorizer function.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequest {
    #[serde(default)]
    pub type_: Option<String>,
    #[serde(default)]
    pub authorization_token: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub method_arn: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayCustomAuthorizerRequestTypeRequest` contains data coming in to a custom API Gateway authorizer function.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerRequestTypeRequest {
    #[serde(default)]
    pub type_: Option<String>,
    /// nolint: stylecheck
    #[serde(default)]
    pub method_arn: Option<String>,
    #[serde(default)]
    pub resource: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(deserialize_with = "http_method::deserialize_optional")]
    #[serde(serialize_with = "http_method::serialize_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub http_method: Option<Method>,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    pub query_string_parameters: QueryMap,
    #[serde(default, deserialize_with = "query_map::serde::standard::deserialize_empty")]
    pub multi_value_query_string_parameters: QueryMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub path_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub stage_variables: HashMap<String, String>,
    pub request_context: ApiGatewayCustomAuthorizerRequestTypeRequestContext,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayCustomAuthorizerResponse` represents the expected format of an API Gateway authorization response.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayCustomAuthorizerResponse<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub principal_id: Option<String>,
    pub policy_document: ApiGatewayCustomAuthorizerPolicy,
    #[serde(bound = "", default)]
    pub context: T1,
    pub usage_identifier_key: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayV2CustomAuthorizerSimpleResponse` represents the simple format of an API Gateway V2 authorization response.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiGatewayV2CustomAuthorizerSimpleResponse<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    pub is_authorized: bool,
    #[serde(bound = "", default)]
    pub context: T1,
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
pub struct ApiGatewayV2CustomAuthorizerIamPolicyResponse<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub principal_id: Option<String>,
    pub policy_document: ApiGatewayCustomAuthorizerPolicy,
    #[serde(bound = "", default)]
    pub context: T1,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ApiGatewayCustomAuthorizerPolicy` represents an IAM policy
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApiGatewayCustomAuthorizerPolicy {
    #[serde(default)]
    pub version: Option<String>,
    pub statement: Vec<IamPolicyStatement>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

fn default_http_method() -> Method {
    Method::GET
}

#[deprecated = "use `ApiGatewayRequestAuthorizer` instead"]
pub type ApiGatewayV2httpRequestContextAuthorizerDescription = ApiGatewayRequestAuthorizer;
#[deprecated = "use `ApiGatewayRequestAuthorizerJwtDescription` instead"]
pub type ApiGatewayV2httpRequestContextAuthorizerJwtDescription = ApiGatewayRequestAuthorizerJwtDescription;
#[deprecated = "use `ApiGatewayRequestAuthorizerIamDescription` instead"]
pub type ApiGatewayV2httpRequestContextAuthorizerIamDescription = ApiGatewayRequestAuthorizerIamDescription;
#[deprecated = "use `ApiGatewayRequestAuthorizerCognitoIdentity` instead"]
pub type ApiGatewayV2httpRequestContextAuthorizerCognitoIdentity = ApiGatewayRequestAuthorizerCognitoIdentity;

impl ApiGatewayRequestAuthorizer {
    fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

fn deserialize_authorizer_fields<'de, D>(deserializer: D) -> Result<ApiGatewayRequestAuthorizer, D::Error>
where
    D: Deserializer<'de>,
{
    let fields: Option<HashMap<String, Value>> = Option::deserialize(deserializer)?;
    let mut authorizer = ApiGatewayRequestAuthorizer::default();
    if let Some(fields) = fields {
        authorizer.fields = fields;
    }

    Ok(authorizer)
}

pub fn serialize_authorizer_fields<S: Serializer>(
    authorizer: &ApiGatewayRequestAuthorizer,
    ser: S,
) -> Result<S::Ok, S::Error> {
    let mut map = ser.serialize_map(Some(authorizer.fields.len()))?;
    for (k, v) in &authorizer.fields {
        map.serialize_entry(k, v)?;
    }
    map.end()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_custom_auth_request_type_request() {
        let data = include_bytes!("../../fixtures/example-apigw-custom-auth-request-type-request.json");
        let parsed: ApiGatewayCustomAuthorizerRequestTypeRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayCustomAuthorizerRequestTypeRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_custom_auth_request_type_request_websocket() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-custom-authorizer-websocket-request.json");
        let parsed: ApiGatewayCustomAuthorizerRequestTypeRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayCustomAuthorizerRequestTypeRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_custom_auth_request() {
        let data = include_bytes!("../../fixtures/example-apigw-custom-auth-request.json");
        let parsed: ApiGatewayCustomAuthorizerRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayCustomAuthorizerRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_custom_auth_response() {
        let data = include_bytes!("../../fixtures/example-apigw-custom-auth-response.json");
        let parsed: ApiGatewayCustomAuthorizerResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayCustomAuthorizerResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_custom_auth_response_with_single_value_action() {
        let data = include_bytes!("../../fixtures/example-apigw-custom-auth-response-with-single-value-action.json");
        let parsed: ApiGatewayCustomAuthorizerResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayCustomAuthorizerResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_custom_auth_response_with_single_value_resource() {
        let data = include_bytes!("../../fixtures/example-apigw-custom-auth-response-with-single-value-resource.json");
        let parsed: ApiGatewayCustomAuthorizerResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayCustomAuthorizerResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_request() {
        let data = include_bytes!("../../fixtures/example-apigw-request.json");
        let parsed: ApiGatewayProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_response() {
        let data = include_bytes!("../../fixtures/example-apigw-response.json");
        let parsed: ApiGatewayProxyResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_request_multi_value_parameters() {
        let data = include_bytes!("../../fixtures/example-apigw-request-multi-value-parameters.json");
        let parsed: ApiGatewayProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);

        assert!(output.contains(r#""multiValueQueryStringParameters":{"name":["me","me2"]}"#));
        assert!(output.contains(r#""queryStringParameters":{"name":"me"}"#));
        assert!(output.contains(r#""headername":["headerValue","headerValue2"]"#));
        assert!(output.contains(r#""headername":"headerValue2""#));
    }

    #[test]
    #[cfg(all(feature = "apigw", feature = "catch-all-fields"))]
    fn example_apigw_request_catch_all() {
        use serde_json::json;

        let data = include_bytes!("../../fixtures/example-apigw-request-catch-all.json");
        let parsed: ApiGatewayProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);

        assert_eq!(parsed.other.get("otherField"), Some(&json!("foobar")));
        assert_eq!(
            parsed.request_context.identity.other.get("otherField"),
            Some(&json!(2345))
        );
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_restapi_openapi_request() {
        let data = include_bytes!("../../fixtures/example-apigw-restapi-openapi-request.json");
        let parsed: ApiGatewayProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_request_iam() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-request-iam.json");
        let parsed: ApiGatewayV2httpRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2httpRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_request_jwt_authorizer() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-request-jwt-authorizer.json");
        let parsed: ApiGatewayV2httpRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2httpRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_request_lambda_authorizer() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-request-lambda-authorizer.json");
        let parsed: ApiGatewayV2httpRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2httpRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_request_multi_value_parameters() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-request-multi-value-parameters.json");
        let parsed: ApiGatewayV2httpRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2httpRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);

        assert!(output.contains(r#""header2":"value1,value2""#));
        assert!(output.contains(r#""queryStringParameters":{"Parameter1":"value1,value2"}"#));
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_request_no_authorizer() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-request-no-authorizer.json");
        let parsed: ApiGatewayV2httpRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2httpRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_websocket_request() {
        let data = include_bytes!("../../fixtures/example-apigw-websocket-request.json");
        let parsed: ApiGatewayWebsocketProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayWebsocketProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_console_test_request() {
        let data = include_bytes!("../../fixtures/example-apigw-console-test-request.json");
        let parsed: ApiGatewayProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_websocket_request_without_method() {
        let data = include_bytes!("../../fixtures/example-apigw-websocket-request-without-method.json");
        let parsed: ApiGatewayWebsocketProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayWebsocketProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_websocket_request_disconnect_route() {
        let data = include_bytes!("../../fixtures/example-apigw-websocket-request-disconnect-route.json");
        let parsed: ApiGatewayWebsocketProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayWebsocketProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_custom_authorizer_v1_request() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-custom-authorizer-v1-request.json");
        let parsed: ApiGatewayV2httpRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2httpRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
        assert_eq!("REQUEST", parsed.kind.unwrap());
        assert_eq!(Method::GET, parsed.http_method);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_custom_authorizer_v2_request() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-custom-authorizer-v2-request.json");
        let parsed: ApiGatewayV2CustomAuthorizerV2Request = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2CustomAuthorizerV2Request = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_custom_authorizer_v2_request_without_cookies() {
        let data = include_bytes!("../../fixtures/example-apigw-v2-custom-authorizer-v2-request-without-cookies.json");
        let parsed: ApiGatewayV2CustomAuthorizerV2Request = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2CustomAuthorizerV2Request = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_v2_custom_authorizer_v2_request_without_identity_source() {
        let data =
            include_bytes!("../../fixtures/example-apigw-v2-custom-authorizer-v2-request-without-identity-source.json");
        let parsed: ApiGatewayV2CustomAuthorizerV2Request = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayV2CustomAuthorizerV2Request = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_console_request() {
        let data = include_bytes!("../../fixtures/example-apigw-console-request.json");
        let parsed: ApiGatewayProxyRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayProxyRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_request_authorizer_fields() {
        let data = include_bytes!("../../fixtures/example-apigw-request.json");
        let parsed: ApiGatewayProxyRequest = serde_json::from_slice(data).unwrap();

        let fields = parsed.request_context.authorizer.fields;
        assert_eq!(Some("admin"), fields.get("principalId").unwrap().as_str());
        assert_eq!(Some(1), fields.get("clientId").unwrap().as_u64());
        assert_eq!(Some("Exata"), fields.get("clientName").unwrap().as_str());
    }

    #[test]
    #[cfg(feature = "apigw")]
    fn example_apigw_custom_auth_response_with_statement_condition() {
        use crate::iam::IamPolicyEffect;

        let data = include_bytes!("../../fixtures/example-apigw-custom-auth-response-with-condition.json");
        let parsed: ApiGatewayCustomAuthorizerResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: ApiGatewayCustomAuthorizerResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);

        let statement = parsed.policy_document.statement.first().unwrap();
        assert_eq!(IamPolicyEffect::Deny, statement.effect);

        let condition = statement.condition.as_ref().unwrap();
        assert_eq!(vec!["xxx"], condition["StringEquals"]["aws:SourceIp"]);
    }
}
