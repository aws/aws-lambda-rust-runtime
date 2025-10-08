use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::custom_serde::{deserialize_lambda_map, deserialize_nullish_boolean};

/// `CognitoEvent` contains data from an event sent from AWS Cognito Sync
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEvent {
    #[serde(default)]
    pub dataset_name: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub dataset_records: HashMap<String, CognitoDatasetRecord>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub identity_id: Option<String>,
    #[serde(default)]
    pub identity_pool_id: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    pub version: i64,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoDatasetRecord` represents a record from an AWS Cognito Sync event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoDatasetRecord {
    #[serde(default)]
    pub new_value: Option<String>,
    #[serde(default)]
    pub old_value: Option<String>,
    #[serde(default)]
    pub op: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsPreSignup` is sent by AWS Cognito User Pools when a user attempts to register
/// (sign up), allowing a Lambda to perform custom validation to accept or deny the registration request
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreSignup {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader<CognitoEventUserPoolsPreSignupTriggerSource>,
    pub request: CognitoEventUserPoolsPreSignupRequest,
    pub response: CognitoEventUserPoolsPreSignupResponse,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum CognitoEventUserPoolsPreSignupTriggerSource {
    #[serde(rename = "PreSignUp_SignUp")]
    #[default]
    SignUp,
    #[serde(rename = "PreSignUp_AdminCreateUser")]
    AdminCreateUser,
    #[serde(rename = "PreSignUp_ExternalProvider")]
    ExternalProvider,
}

/// `CognitoEventUserPoolsPreAuthentication` is sent by AWS Cognito User Pools when a user submits their information
/// to be authenticated, allowing you to perform custom validations to accept or deny the sign in request.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreAuthentication {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header:
        CognitoEventUserPoolsHeader<CognitoEventUserPoolsPreAuthenticationTriggerSource>,
    pub request: CognitoEventUserPoolsPreAuthenticationRequest,
    pub response: CognitoEventUserPoolsPreAuthenticationResponse,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum CognitoEventUserPoolsPreAuthenticationTriggerSource {
    #[serde(rename = "PreAuthentication_Authentication")]
    #[default]
    Authentication,
}

/// `CognitoEventUserPoolsPostConfirmation` is sent by AWS Cognito User Pools after a user is confirmed,
/// allowing the Lambda to send custom messages or add custom logic.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPostConfirmation<T = CognitoEventUserPoolsPostConfirmationResponse>
where
    T: DeserializeOwned,
    T: Serialize,
{
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header:
        CognitoEventUserPoolsHeader<CognitoEventUserPoolsPostConfirmationTriggerSource>,
    pub request: CognitoEventUserPoolsPostConfirmationRequest,
    #[serde(bound = "")]
    pub response: T,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum CognitoEventUserPoolsPostConfirmationTriggerSource {
    #[serde(rename = "PostConfirmation_ConfirmForgotPassword")]
    ConfirmForgotPassword,
    #[serde(rename = "PostConfirmation_ConfirmSignUp")]
    #[default]
    ConfirmSignUp,
}

/// `CognitoEventUserPoolsPreTokenGen` is sent by AWS Cognito User Pools when a user attempts to retrieve
/// credentials, allowing a Lambda to perform insert, suppress or override claims
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreTokenGen {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader<CognitoEventUserPoolsPreTokenGenTriggerSource>,
    pub request: CognitoEventUserPoolsPreTokenGenRequest,
    pub response: CognitoEventUserPoolsPreTokenGenResponse,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum CognitoEventUserPoolsPreTokenGenTriggerSource {
    #[serde(rename = "TokenGeneration_HostedAuth")]
    HostedAuth,
    #[serde(rename = "TokenGeneration_Authentication")]
    #[default]
    Authentication,
    #[serde(rename = "TokenGeneration_NewPasswordChallenge")]
    NewPasswordChallenge,
    #[serde(rename = "TokenGeneration_AuthenticateDevice")]
    AuthenticateDevice,
    #[serde(rename = "TokenGeneration_RefreshTokens")]
    RefreshTokens,
}

/// `CognitoEventUserPoolsPostAuthentication` is sent by AWS Cognito User Pools after a user is authenticated,
/// allowing the Lambda to add custom logic.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPostAuthentication {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header:
        CognitoEventUserPoolsHeader<CognitoEventUserPoolsPostAuthenticationTriggerSource>,
    pub request: CognitoEventUserPoolsPostAuthenticationRequest,
    pub response: CognitoEventUserPoolsPostAuthenticationResponse,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum CognitoEventUserPoolsPostAuthenticationTriggerSource {
    #[serde(rename = "PostAuthentication_Authentication")]
    #[default]
    Authentication,
}

/// `CognitoEventUserPoolsMigrateUser` is sent by AWS Cognito User Pools when a user does not exist in the
/// user pool at the time of sign-in with a password, or in the forgot-password flow.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsMigrateUser {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader<CognitoEventUserPoolsMigrateUserTriggerSource>,
    #[serde(rename = "request")]
    pub cognito_event_user_pools_migrate_user_request: CognitoEventUserPoolsMigrateUserRequest,
    #[serde(rename = "response")]
    pub cognito_event_user_pools_migrate_user_response: CognitoEventUserPoolsMigrateUserResponse,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum CognitoEventUserPoolsMigrateUserTriggerSource {
    #[serde(rename = "UserMigration_Authentication")]
    #[default]
    Authentication,
    #[serde(rename = "UserMigration_ForgotPassword")]
    ForgotPassword,
}

/// `CognitoEventUserPoolsCallerContext` contains information about the caller
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCallerContext {
    #[serde(default)]
    #[serde(rename = "awsSdkVersion")]
    pub awssdk_version: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsHeader` contains common data from events sent by AWS Cognito User Pools
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsHeader<T> {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub trigger_source: Option<T>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub user_pool_id: Option<String>,
    pub caller_context: CognitoEventUserPoolsCallerContext,
    #[serde(default)]
    pub user_name: Option<String>,
    // no `other` catch-all, because this struct is itself #[serde(flatten)]-ed
    // into a different struct that contains an `other` catch-all, so any
    // additional fields will be caught there instead
}

/// `CognitoEventUserPoolsPreSignupRequest` contains the request portion of a PreSignup event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreSignupRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub validation_data: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsPreSignupResponse` contains the response portion of a PreSignup event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreSignupResponse {
    pub auto_confirm_user: bool,
    pub auto_verify_email: bool,
    pub auto_verify_phone: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsPreAuthenticationRequest` contains the request portion of a PreAuthentication event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreAuthenticationRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub validation_data: HashMap<String, String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsPreAuthenticationResponse` contains the response portion of a PreAuthentication event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CognitoEventUserPoolsPreAuthenticationResponse {
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}
/// `CognitoEventUserPoolsPostConfirmationRequest` contains the request portion of a PostConfirmation event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPostConfirmationRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsPostConfirmationResponse` contains the response portion of a PostConfirmation event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CognitoEventUserPoolsPostConfirmationResponse {
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsPreTokenGenRequest` contains request portion of PreTokenGen event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreTokenGenRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    pub group_configuration: GroupConfiguration,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsPreTokenGenResponse` contains the response portion of  a PreTokenGen event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreTokenGenResponse {
    pub claims_override_details: Option<ClaimsOverrideDetails>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsPreTokenGenV2` is sent by AWS Cognito User Pools when a user attempts to retrieve
/// credentials, allowing a Lambda to perform insert, suppress or override claims.  This is the Version 2 Payload
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreTokenGenV2 {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader<CognitoEventUserPoolsPreTokenGenTriggerSource>,
    pub request: CognitoEventUserPoolsPreTokenGenRequestV2,
    pub response: CognitoEventUserPoolsPreTokenGenResponseV2,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsPreTokenGenRequestV2` contains request portion of PreTokenGenV2 event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPreTokenGenRequestV2 {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    pub group_configuration: GroupConfiguration,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
    pub scopes: Vec<String>,
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
pub struct CognitoEventUserPoolsPreTokenGenResponseV2 {
    pub claims_and_scope_override_details: Option<ClaimsAndScopeOverrideDetailsV2>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ClaimsAndScopeOverrideDetailsV2` allows lambda to add, suppress or override claims in the token
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimsAndScopeOverrideDetailsV2 {
    pub group_override_details: GroupConfiguration,
    pub id_token_generation: Option<CognitoIdTokenGenerationV2>,
    pub access_token_generation: Option<CognitoAccessTokenGenerationV2>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoIdTokenGenerationV2` allows lambda to customize the ID Token before generation
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoIdTokenGenerationV2 {
    pub claims_to_add_or_override: HashMap<String, String>,
    pub claims_to_suppress: Vec<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoAccessTokenGenerationV2` allows lambda to customize the Access Token before generation
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoAccessTokenGenerationV2 {
    pub claims_to_add_or_override: HashMap<String, String>,
    pub claims_to_suppress: Vec<String>,
    pub scopes_to_add: Vec<String>,
    pub scopes_to_suppress: Vec<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsPostAuthenticationRequest` contains the request portion of a PostAuthentication event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsPostAuthenticationRequest {
    pub new_device_used: bool,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsPostAuthenticationResponse` contains the response portion of a PostAuthentication event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct CognitoEventUserPoolsPostAuthenticationResponse {
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}
/// `CognitoEventUserPoolsMigrateUserRequest` contains the request portion of a MigrateUser event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsMigrateUserRequest {
    #[serde(default)]
    pub password: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub validation_data: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsMigrateUserResponse` contains the response portion of a MigrateUser event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsMigrateUserResponse {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(default)]
    pub final_user_status: Option<String>,
    #[serde(default)]
    pub message_action: Option<String>,
    #[serde(default)]
    pub desired_delivery_mediums: Option<Vec<String>>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub force_alias_creation: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ClaimsOverrideDetails` allows lambda to add, suppress or override claims in the token
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaimsOverrideDetails {
    pub group_override_details: GroupConfiguration,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub claims_to_add_or_override: HashMap<String, String>,
    pub claims_to_suppress: Vec<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `GroupConfiguration` allows lambda to override groups, roles and set a preferred role
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupConfiguration {
    pub groups_to_override: Vec<String>,
    pub iam_roles_to_override: Vec<String>,
    pub preferred_role: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsChallengeResult` represents a challenge that is presented to the user in the authentication
/// process that is underway, along with the corresponding result.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsChallengeResult {
    #[serde(default)]
    pub challenge_name: Option<String>,
    pub challenge_result: bool,
    #[serde(default)]
    pub challenge_metadata: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsDefineAuthChallengeRequest` defines auth challenge request parameters
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsDefineAuthChallengeRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    pub session: Vec<Option<CognitoEventUserPoolsChallengeResult>>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
    #[serde(default)]
    pub user_not_found: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsDefineAuthChallengeResponse` defines auth challenge response parameters
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsDefineAuthChallengeResponse {
    #[serde(default)]
    pub challenge_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub issue_tokens: bool,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub fail_authentication: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsDefineAuthChallenge` sent by AWS Cognito User Pools to initiate custom authentication flow
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsDefineAuthChallenge {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header:
        CognitoEventUserPoolsHeader<CognitoEventUserPoolsDefineAuthChallengeTriggerSource>,
    pub request: CognitoEventUserPoolsDefineAuthChallengeRequest,
    pub response: CognitoEventUserPoolsDefineAuthChallengeResponse,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum CognitoEventUserPoolsDefineAuthChallengeTriggerSource {
    #[serde(rename = "DefineAuthChallenge_Authentication")]
    #[default]
    Authentication,
}

/// `CognitoEventUserPoolsCreateAuthChallengeRequest` defines create auth challenge request parameters
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCreateAuthChallengeRequest {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(default)]
    pub challenge_name: Option<String>,
    pub session: Vec<Option<CognitoEventUserPoolsChallengeResult>>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
    #[serde(default)]
    pub user_not_found: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsCreateAuthChallengeResponse` defines create auth challenge response parameters
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCreateAuthChallengeResponse {
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub public_challenge_parameters: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub private_challenge_parameters: HashMap<String, String>,
    #[serde(default)]
    pub challenge_metadata: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsCreateAuthChallenge` sent by AWS Cognito User Pools to create a challenge to present to the user
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCreateAuthChallenge {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header:
        CognitoEventUserPoolsHeader<CognitoEventUserPoolsCreateAuthChallengeTriggerSource>,
    pub request: CognitoEventUserPoolsCreateAuthChallengeRequest,
    pub response: CognitoEventUserPoolsCreateAuthChallengeResponse,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum CognitoEventUserPoolsCreateAuthChallengeTriggerSource {
    #[serde(rename = "CreateAuthChallenge_Authentication")]
    #[default]
    Authentication,
}

/// `CognitoEventUserPoolsVerifyAuthChallengeRequest` defines verify auth challenge request parameters
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsVerifyAuthChallengeRequest<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub user_attributes: HashMap<String, String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub private_challenge_parameters: HashMap<String, String>,
    #[serde(bound = "")]
    pub challenge_answer: Option<T1>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
    #[serde(default)]
    pub user_not_found: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsVerifyAuthChallengeResponse` defines verify auth challenge response parameters
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsVerifyAuthChallengeResponse {
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub answer_correct: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsVerifyAuthChallenge` sent by AWS Cognito User Pools to verify if the response from the end user
/// for a custom Auth Challenge is valid or not
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsVerifyAuthChallenge {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header:
        CognitoEventUserPoolsHeader<CognitoEventUserPoolsVerifyAuthChallengeTriggerSource>,
    pub request: CognitoEventUserPoolsVerifyAuthChallengeRequest,
    pub response: CognitoEventUserPoolsVerifyAuthChallengeResponse,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum CognitoEventUserPoolsVerifyAuthChallengeTriggerSource {
    #[serde(rename = "VerifyAuthChallengeResponse_Authentication")]
    #[default]
    Authentication,
}

/// `CognitoEventUserPoolsCustomMessage` is sent by AWS Cognito User Pools before a verification or MFA message is sent,
/// allowing a user to customize the message dynamically.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCustomMessage {
    #[serde(rename = "CognitoEventUserPoolsHeader")]
    #[serde(flatten)]
    pub cognito_event_user_pools_header: CognitoEventUserPoolsHeader<CognitoEventUserPoolsCustomMessageTriggerSource>,
    pub request: CognitoEventUserPoolsCustomMessageRequest,
    pub response: CognitoEventUserPoolsCustomMessageResponse,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Default)]
pub enum CognitoEventUserPoolsCustomMessageTriggerSource {
    #[serde(rename = "CustomMessage_SignUp")]
    #[default]
    SignUp,
    #[serde(rename = "CustomMessage_AdminCreateUser")]
    AdminCreateUser,
    #[serde(rename = "CustomMessage_ResendCode")]
    ResendCode,
    #[serde(rename = "CustomMessage_ForgotPassword")]
    ForgotPassword,
    #[serde(rename = "CustomMessage_UpdateUserAttribute")]
    UpdateUserAttribute,
    #[serde(rename = "CustomMessage_VerifyUserAttribute")]
    VerifyUserAttribute,
    #[serde(rename = "CustomMessage_Authentication")]
    Authentication,
}

/// `CognitoEventUserPoolsCustomMessageRequest` contains the request portion of a CustomMessage event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCustomMessageRequest<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub user_attributes: HashMap<String, T1>,
    #[serde(default)]
    pub code_parameter: Option<String>,
    #[serde(default)]
    pub username_parameter: Option<String>,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    pub client_metadata: HashMap<String, String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CognitoEventUserPoolsCustomMessageResponse` contains the response portion of a CustomMessage event
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CognitoEventUserPoolsCustomMessageResponse {
    #[serde(default)]
    pub sms_message: Option<String>,
    #[serde(default)]
    pub email_message: Option<String>,
    #[serde(default)]
    pub email_subject: Option<String>,
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
    #[cfg(feature = "cognito")]
    fn example_cognito_event() {
        let data = include_bytes!("../../fixtures/example-cognito-event.json");
        let parsed: CognitoEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_create_auth_challenge() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-create-auth-challenge.json");
        let parsed: CognitoEventUserPoolsCreateAuthChallenge = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsCreateAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_create_auth_challenge_user_not_found() {
        let data =
            include_bytes!("../../fixtures/example-cognito-event-userpools-create-auth-challenge-user-not-found.json");
        let parsed: CognitoEventUserPoolsCreateAuthChallenge = serde_json::from_slice(data).unwrap();

        assert!(parsed.request.user_not_found);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsCreateAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_custommessage() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-custommessage.json");
        let parsed: CognitoEventUserPoolsCustomMessage = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsCustomMessage = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_define_auth_challenge() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-define-auth-challenge.json");
        let parsed: CognitoEventUserPoolsDefineAuthChallenge = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsDefineAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_define_auth_challenge_optional_response_fields() {
        let data = include_bytes!(
            "../../fixtures/example-cognito-event-userpools-define-auth-challenge-optional-response-fields.json"
        );
        let parsed: CognitoEventUserPoolsDefineAuthChallenge = serde_json::from_slice(data).unwrap();

        assert!(!parsed.response.fail_authentication);
        assert!(!parsed.response.issue_tokens);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsDefineAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_define_auth_challenge_user_not_found() {
        let data =
            include_bytes!("../../fixtures/example-cognito-event-userpools-define-auth-challenge-user-not-found.json");
        let parsed: CognitoEventUserPoolsDefineAuthChallenge = serde_json::from_slice(data).unwrap();

        assert!(parsed.request.user_not_found);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsDefineAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_migrateuser() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-migrateuser.json");
        let parsed: CognitoEventUserPoolsMigrateUser = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsMigrateUser = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_postauthentication() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-postauthentication.json");
        let parsed: CognitoEventUserPoolsPostAuthentication = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPostAuthentication = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_postconfirmation() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-postconfirmation.json");
        let parsed: CognitoEventUserPoolsPostConfirmation = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPostConfirmation = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_preauthentication() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-preauthentication.json");
        let parsed: CognitoEventUserPoolsPreAuthentication = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPreAuthentication = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_presignup() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-presignup.json");
        let parsed: CognitoEventUserPoolsPreSignup = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPreSignup = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_pretokengen_incoming() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-pretokengen-incoming.json");
        let parsed: CognitoEventUserPoolsPreTokenGen = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPreTokenGen = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_pretokengen_v2_incoming() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-pretokengen-v2-incoming.json");
        let parsed: CognitoEventUserPoolsPreTokenGenV2 = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPreTokenGenV2 = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_pretokengen() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-pretokengen.json");
        let parsed: CognitoEventUserPoolsPreTokenGen = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPreTokenGen = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_v2_pretokengen() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-pretokengen-v2.json");
        let parsed: CognitoEventUserPoolsPreTokenGenV2 = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsPreTokenGenV2 = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_verify_auth_challenge() {
        let data = include_bytes!("../../fixtures/example-cognito-event-userpools-verify-auth-challenge.json");
        let parsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_verify_auth_challenge_optional_answer_correct() {
        let data = include_bytes!(
            "../../fixtures/example-cognito-event-userpools-verify-auth-challenge-optional-answer-correct.json"
        );
        let parsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(data).unwrap();

        assert!(!parsed.response.answer_correct);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_verify_auth_challenge_null_answer_correct() {
        let data = include_bytes!(
            "../../fixtures/example-cognito-event-userpools-verify-auth-challenge-null-answer-correct.json"
        );
        let parsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(data).unwrap();

        assert!(!parsed.response.answer_correct);

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "cognito")]
    fn example_cognito_event_userpools_verify_auth_challenge_user_not_found() {
        let data =
            include_bytes!("../../fixtures/example-cognito-event-userpools-verify-auth-challenge-user-not-found.json");
        let parsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(data).unwrap();

        assert!(parsed.request.user_not_found);

        let output: String = serde_json::to_string_pretty(&parsed).unwrap();
        println!("output is: {output}");
        let reparsed: CognitoEventUserPoolsVerifyAuthChallenge = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}

#[cfg(test)]
#[cfg(feature = "cognito")]
mod trigger_source_tests {
    use super::*;

    fn gen_header(trigger_source: &str) -> String {
        format!(
            r#"
{{
    "version": "1",
    "triggerSource": "{trigger_source}",
    "region": "region",
    "userPoolId": "userPoolId",
    "userName": "userName",
    "callerContext": {{
        "awsSdkVersion": "calling aws sdk with version",
        "clientId": "apps client id"
    }}
}}"#
        )
    }

    #[test]
    fn pre_sign_up() {
        let possible_triggers = [
            "PreSignUp_AdminCreateUser",
            "PreSignUp_AdminCreateUser",
            "PreSignUp_ExternalProvider",
        ];
        possible_triggers.into_iter().for_each(|trigger| {
            let header = gen_header(trigger);
            let parsed: CognitoEventUserPoolsHeader<CognitoEventUserPoolsPreSignupTriggerSource> =
                serde_json::from_str(&header).unwrap();
            let output: String = serde_json::to_string(&parsed).unwrap();
            let reparsed: CognitoEventUserPoolsHeader<_> = serde_json::from_slice(output.as_bytes()).unwrap();
            assert_eq!(parsed, reparsed);
        });
    }

    #[test]
    fn pre_authentication() {
        let possible_triggers = ["PreAuthentication_Authentication"];
        possible_triggers.into_iter().for_each(|trigger| {
            let header = gen_header(trigger);
            let parsed: CognitoEventUserPoolsHeader<CognitoEventUserPoolsPreAuthenticationTriggerSource> =
                serde_json::from_str(&header).unwrap();
            let output: String = serde_json::to_string(&parsed).unwrap();
            let reparsed: CognitoEventUserPoolsHeader<_> = serde_json::from_slice(output.as_bytes()).unwrap();
            assert_eq!(parsed, reparsed);
        });
    }
    #[test]
    fn post_confirmation() {
        let possible_triggers = [
            "PostConfirmation_ConfirmForgotPassword",
            "PostConfirmation_ConfirmSignUp",
        ];

        possible_triggers.into_iter().for_each(|trigger| {
            let header = gen_header(trigger);
            let parsed: CognitoEventUserPoolsHeader<CognitoEventUserPoolsPostConfirmationTriggerSource> =
                serde_json::from_str(&header).unwrap();
            let output: String = serde_json::to_string(&parsed).unwrap();
            let reparsed: CognitoEventUserPoolsHeader<_> = serde_json::from_slice(output.as_bytes()).unwrap();
            assert_eq!(parsed, reparsed);
        });
    }
    #[test]
    fn post_authentication() {
        let possible_triggers = ["PostAuthentication_Authentication"];

        possible_triggers.into_iter().for_each(|trigger| {
            let header = gen_header(trigger);
            let parsed: CognitoEventUserPoolsHeader<CognitoEventUserPoolsPostAuthenticationTriggerSource> =
                serde_json::from_str(&header).unwrap();
            let output: String = serde_json::to_string(&parsed).unwrap();
            let reparsed: CognitoEventUserPoolsHeader<_> = serde_json::from_slice(output.as_bytes()).unwrap();
            assert_eq!(parsed, reparsed);
        });
    }
    #[test]
    fn define_auth_challenge() {
        let possible_triggers = ["DefineAuthChallenge_Authentication"];

        possible_triggers.into_iter().for_each(|trigger| {
            let header = gen_header(trigger);
            let parsed: CognitoEventUserPoolsHeader<CognitoEventUserPoolsDefineAuthChallengeTriggerSource> =
                serde_json::from_str(&header).unwrap();
            let output: String = serde_json::to_string(&parsed).unwrap();
            let reparsed: CognitoEventUserPoolsHeader<_> = serde_json::from_slice(output.as_bytes()).unwrap();
            assert_eq!(parsed, reparsed);
        });
    }

    #[test]
    fn create_auth_challenge() {
        let possible_triggers = ["CreateAuthChallenge_Authentication"];

        possible_triggers.into_iter().for_each(|trigger| {
            let header = gen_header(trigger);
            let parsed: CognitoEventUserPoolsHeader<CognitoEventUserPoolsCreateAuthChallengeTriggerSource> =
                serde_json::from_str(&header).unwrap();
            let output: String = serde_json::to_string(&parsed).unwrap();
            let reparsed: CognitoEventUserPoolsHeader<_> = serde_json::from_slice(output.as_bytes()).unwrap();
            assert_eq!(parsed, reparsed);
        });
    }
    #[test]
    fn verify_auth_challenge() {
        let possible_triggers = ["VerifyAuthChallengeResponse_Authentication"];

        possible_triggers.into_iter().for_each(|trigger| {
            let header = gen_header(trigger);
            let parsed: CognitoEventUserPoolsHeader<CognitoEventUserPoolsVerifyAuthChallengeTriggerSource> =
                serde_json::from_str(&header).unwrap();
            let output: String = serde_json::to_string(&parsed).unwrap();
            let reparsed: CognitoEventUserPoolsHeader<_> = serde_json::from_slice(output.as_bytes()).unwrap();
            assert_eq!(parsed, reparsed);
        });
    }
    #[test]
    fn pre_token_generation() {
        let possible_triggers = [
            "TokenGeneration_HostedAuth",
            "TokenGeneration_Authentication",
            "TokenGeneration_NewPasswordChallenge",
            "TokenGeneration_AuthenticateDevice",
            "TokenGeneration_RefreshTokens",
        ];

        possible_triggers.into_iter().for_each(|trigger| {
            let header = gen_header(trigger);
            let parsed: CognitoEventUserPoolsHeader<CognitoEventUserPoolsPreTokenGenTriggerSource> =
                serde_json::from_str(&header).unwrap();
            let output: String = serde_json::to_string(&parsed).unwrap();
            let reparsed: CognitoEventUserPoolsHeader<_> = serde_json::from_slice(output.as_bytes()).unwrap();
            assert_eq!(parsed, reparsed);
        });
    }
    #[test]
    fn user_migration() {
        let possible_triggers = ["UserMigration_Authentication", "UserMigration_ForgotPassword"];

        possible_triggers.into_iter().for_each(|trigger| {
            let header = gen_header(trigger);
            let parsed: CognitoEventUserPoolsHeader<CognitoEventUserPoolsMigrateUserTriggerSource> =
                serde_json::from_str(&header).unwrap();
            let output: String = serde_json::to_string(&parsed).unwrap();
            let reparsed: CognitoEventUserPoolsHeader<_> = serde_json::from_slice(output.as_bytes()).unwrap();
            assert_eq!(parsed, reparsed);
        });
    }
    #[test]
    fn custom_message() {
        let possible_triggers = [
            "CustomMessage_SignUp",
            "CustomMessage_AdminCreateUser",
            "CustomMessage_ResendCode",
            "CustomMessage_ForgotPassword",
            "CustomMessage_UpdateUserAttribute",
            "CustomMessage_VerifyUserAttribute",
            "CustomMessage_Authentication",
        ];

        possible_triggers.into_iter().for_each(|trigger| {
            let header = gen_header(trigger);
            let parsed: CognitoEventUserPoolsHeader<CognitoEventUserPoolsCustomMessageTriggerSource> =
                serde_json::from_str(&header).unwrap();
            let output: String = serde_json::to_string(&parsed).unwrap();
            let reparsed: CognitoEventUserPoolsHeader<_> = serde_json::from_slice(output.as_bytes()).unwrap();
            assert_eq!(parsed, reparsed);
        });
    }
}
