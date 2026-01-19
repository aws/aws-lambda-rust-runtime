#[cfg(feature = "builders")]
use bon::Builder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::custom_serde::deserialize_lambda_map;

/// Deprecated: `AppSyncResolverTemplate` does not represent resolver events sent by AppSync. Instead directly model your input schema, or use `map[string]string`, `json.RawMessage`,` interface{}`, etc..
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncResolverTemplate<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub version: Option<String>,
    pub operation: AppSyncOperation,
    #[serde(bound = "")]
    pub payload: Option<T1>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `AppSyncIamIdentity` contains information about the caller authed via IAM.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncIamIdentity {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub cognito_identity_auth_provider: Option<String>,
    #[serde(default)]
    pub cognito_identity_auth_type: Option<String>,
    #[serde(default)]
    pub cognito_identity_pool_id: Option<String>,
    #[serde(default)]
    pub cognito_identity_id: Option<String>,
    pub source_ip: Vec<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub user_arn: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `AppSyncCognitoIdentity` contains information about the caller authed via Cognito.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::should_implement_trait)]
pub struct AppSyncCognitoIdentity<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    pub sub: Option<String>,
    #[serde(default)]
    pub issuer: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub claims: HashMap<String, T1>,
    pub source_ip: Vec<String>,
    #[serde(default)]
    pub default_auth_strategy: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

pub type AppSyncOperation = String;

/// `AppSyncLambdaAuthorizerRequest` contains an authorization request from AppSync.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncLambdaAuthorizerRequest {
    #[serde(default)]
    pub authorization_token: Option<String>,
    pub request_context: AppSyncLambdaAuthorizerRequestContext,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `AppSyncLambdaAuthorizerRequestContext` contains the parameters of the AppSync invocation which triggered
/// this authorization request.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncLambdaAuthorizerRequestContext<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(default)]
    #[serde(rename = "apiId")]
    pub apiid: Option<String>,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub query_string: Option<String>,
    #[serde(default)]
    pub operation_name: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub variables: HashMap<String, T1>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `AppSyncLambdaAuthorizerResponse` represents the expected format of an authorization response to AppSync.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncLambdaAuthorizerResponse<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    pub is_authorized: bool,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub resolver_context: HashMap<String, T1>,
    pub denied_fields: Option<Vec<String>>,
    pub ttl_override: Option<i64>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `AppSyncDirectResolverEvent` represents the default payload structure sent by AWS AppSync
/// when using **Direct Lambda Resolvers** (i.e., when both request and response mapping
/// templates are disabled).
///
/// This structure includes the full AppSync **Context object**, as described in the
/// [AppSync Direct Lambda resolver reference](https://docs.aws.amazon.com/appsync/latest/devguide/direct-lambda-reference.html).
///
/// It is recommended when working without VTL templates and relying on the standard
/// AppSync-to-Lambda event format.
///
/// See also:
/// - [AppSync resolver mapping template context reference](https://docs.aws.amazon.com/appsync/latest/devguide/resolver-context-reference.html)
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct AppSyncDirectResolverEvent<TArguments = Value, TSource = Value, TStash = Value>
where
    TArguments: Serialize + DeserializeOwned,
    TSource: Serialize + DeserializeOwned,
    TStash: Serialize + DeserializeOwned,
{
    #[serde(bound = "")]
    pub arguments: Option<TArguments>,
    pub identity: Option<AppSyncIdentity>,
    #[serde(bound = "")]
    pub source: Option<TSource>,
    pub request: AppSyncRequest,
    pub info: AppSyncInfo,
    #[serde(default)]
    pub prev: Option<AppSyncPrevResult>,
    #[serde(bound = "")]
    pub stash: TStash,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `AppSyncRequest` contains request-related metadata for a resolver invocation,
/// including client-sent headers and optional custom domain name.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub headers: HashMap<String, Option<String>>,
    #[serde(default)]
    pub domain_name: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `AppSyncInfo` contains metadata about the current GraphQL field being resolved.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncInfo<T = Value>
where
    T: Serialize + DeserializeOwned,
{
    #[serde(default)]
    pub selection_set_list: Vec<String>,
    #[serde(rename = "selectionSetGraphQL")]
    pub selection_set_graphql: String,
    pub parent_type_name: String,
    pub field_name: String,
    #[serde(bound = "")]
    pub variables: T,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `AppSyncPrevResult` contains the result of the previous step in a pipeline resolver.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct AppSyncPrevResult<T = Value>
where
    T: Serialize + DeserializeOwned,
{
    #[serde(bound = "")]
    pub result: T,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `AppSyncIdentity` represents the identity of the caller as determined by the
/// configured AppSync authorization mechanism (IAM, Cognito, OIDC, or Lambda).
#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum AppSyncIdentity {
    IAM(AppSyncIamIdentity),
    Cognito(AppSyncCognitoIdentity),
    OIDC(AppSyncIdentityOIDC),
    Lambda(AppSyncIdentityLambda),
}

impl Default for AppSyncIdentity {
    fn default() -> Self {
        AppSyncIdentity::IAM(AppSyncIamIdentity::default())
    }
}

/// `AppSyncIdentityOIDC` represents identity information when using OIDC-based authorization.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[allow(clippy::should_implement_trait)]
pub struct AppSyncIdentityOIDC<T = Value>
where
    T: Serialize + DeserializeOwned,
{
    #[serde(bound = "")]
    pub claims: T,
    pub issuer: String,
    pub sub: String,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// `AppSyncIdentityLambda` represents identity information when using AWS Lambda
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncIdentityLambda<T = Value>
where
    T: Serialize + DeserializeOwned,
{
    #[serde(bound = "")]
    pub resolver_context: T,
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
    #[cfg(feature = "appsync")]
    fn example_appsync_identity_cognito() {
        let data = include_bytes!("../../fixtures/example-appsync-identity-cognito.json");
        let parsed: AppSyncCognitoIdentity = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AppSyncCognitoIdentity = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "appsync")]
    fn example_appsync_identity_iam() {
        let data = include_bytes!("../../fixtures/example-appsync-identity-iam.json");
        let parsed: AppSyncIamIdentity = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AppSyncIamIdentity = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "appsync")]
    fn example_appsync_lambda_auth_request() {
        let data = include_bytes!("../../fixtures/example-appsync-lambda-auth-request.json");
        let parsed: AppSyncLambdaAuthorizerRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AppSyncLambdaAuthorizerRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "appsync")]
    fn example_appsync_lambda_auth_response() {
        let data = include_bytes!("../../fixtures/example-appsync-lambda-auth-response.json");
        let parsed: AppSyncLambdaAuthorizerResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AppSyncLambdaAuthorizerResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "appsync")]
    fn example_appsync_direct_resolver() {
        let data = include_bytes!("../../fixtures/example-appsync-direct-resolver.json");
        let parsed: AppSyncDirectResolverEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AppSyncDirectResolverEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
