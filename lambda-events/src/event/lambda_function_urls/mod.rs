#[cfg(feature = "builders")]
use bon::Builder;
use http::HeaderMap;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;
use std::collections::HashMap;

use crate::custom_serde::{deserialize_lambda_map, serialize_headers};

/// `LambdaFunctionUrlRequest` contains data coming from the HTTP request to a Lambda Function URL.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LambdaFunctionUrlRequest {
    /// Version is expected to be `"2.0"`
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub raw_path: Option<String>,
    #[serde(default)]
    pub raw_query_string: Option<String>,
    pub cookies: Option<Vec<String>>,
    #[serde(deserialize_with = "http_serde::header_map::deserialize", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub query_string_parameters: HashMap<String, String>,
    pub request_context: LambdaFunctionUrlRequestContext,
    pub body: Option<String>,
    pub is_base64_encoded: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `LambdaFunctionUrlRequestContext` contains the information to identify the AWS account and resources invoking the Lambda function.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LambdaFunctionUrlRequestContext {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    pub authorizer: Option<LambdaFunctionUrlRequestContextAuthorizerDescription>,
    /// APIID is the Lambda URL ID
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
    /// DomainName is of the format `"<url-id>.lambda-url.<region>.on.aws"`
    #[serde(default)]
    pub domain_name: Option<String>,
    /// DomainPrefix is the Lambda URL ID
    #[serde(default)]
    pub domain_prefix: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    pub time_epoch: i64,
    pub http: LambdaFunctionUrlRequestContextHttpDescription,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `LambdaFunctionUrlRequestContextAuthorizerDescription` contains authorizer information for the request context.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LambdaFunctionUrlRequestContextAuthorizerDescription {
    pub iam: Option<LambdaFunctionUrlRequestContextAuthorizerIamDescription>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `LambdaFunctionUrlRequestContextAuthorizerIamDescription` contains IAM information for the request context.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LambdaFunctionUrlRequestContextAuthorizerIamDescription {
    #[serde(default)]
    pub access_key: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub caller_id: Option<String>,
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
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `LambdaFunctionUrlRequestContextHttpDescription` contains HTTP information for the request context.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LambdaFunctionUrlRequestContextHttpDescription {
    #[serde(default)]
    pub method: Option<String>,
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
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `LambdaFunctionUrlResponse` configures the HTTP response to be returned by Lambda Function URL for the request.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LambdaFunctionUrlResponse {
    pub status_code: i64,
    #[serde(deserialize_with = "http_serde::header_map::deserialize", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(default)]
    pub body: Option<String>,
    pub is_base64_encoded: bool,
    pub cookies: Vec<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}
