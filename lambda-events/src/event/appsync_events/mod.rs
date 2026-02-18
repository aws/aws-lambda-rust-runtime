use std::{collections::HashMap, fmt};

use crate::custom_serde::{
    deserialize_headers, deserialize_lambda_map, deserialize_stringified_json, serialize_headers,
    serialize_stringified_json,
};
use http::HeaderMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

/// `AppSyncEventsLambdaAuthorizerRequest` contains an authorization request from AppSync Events.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncEventsLambdaAuthorizerRequest {
    #[serde(default)]
    pub authorization_token: Option<String>,
    pub request_context: AppSyncEventsLambdaAuthorizerRequestContext,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
}

/// `AppSyncEventsLambdaAuthorizerRequestContext` contains the parameters of the AppSync Events invocation which triggered
/// this authorization request.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncEventsLambdaAuthorizerRequestContext {
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub api_id: Option<String>,
    #[serde(default)]
    pub operation: Option<AppSyncEventsOperation>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_namespace_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
}

/// `AppSyncEventsOperation` represent all the possible operations which
/// triggered this authorization request.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppSyncEventsOperation {
    EventConnect,
    EventSubscribe,
    EventPublish,
}

impl fmt::Display for AppSyncEventsOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = match self {
            AppSyncEventsOperation::EventConnect => "EVENT_CONNECT",
            AppSyncEventsOperation::EventSubscribe => "EVENT_SUBSCRIBE",
            AppSyncEventsOperation::EventPublish => "EVENT_PUBLISH",
        };

        write!(f, "{val}")
    }
}

/// `AppSyncEventsLambdaAuthorizerResponse` represents the expected format of an authorization response to AppSync Events.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncEventsLambdaAuthorizerResponse<T1 = Value>
where
    T1: DeserializeOwned + Serialize,
{
    pub is_authorized: bool,
    #[serde(deserialize_with = "deserialize_lambda_map")]
    #[serde(default)]
    #[serde(bound = "")]
    pub handler_context: HashMap<String, T1>,
    pub ttl_override: Option<i64>,
}

/// `AppSyncEventsWebscoketMessage` represents all possible messages which can be sent between
/// AppSync Events and a connected websocket client.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum AppSyncEventsWebsocketMessage<T1 = Value>
where
    T1: DeserializeOwned + Serialize,
{
    ConnectionInit,
    ConnectionAck(AppSyncEventsConnectionAckMessage),
    #[serde(rename = "ka")]
    KeepAlive,
    #[serde(bound = "")]
    Subscribe(AppSyncEventsSubscribeMessage),
    SubscribeSuccess(AppSyncEventsSubscribeSuccessMessage),
    SubscribeError(AppSyncEventsErrorMessage),
    #[serde(bound = "")]
    Data(AppSyncEventsDataMessage<T1>),
    BroadcastError(AppSyncEventsErrorMessage),
    Unsubscribe(AppSyncEventsUnsubscribeMessage),
    UnsubscribeSuccess(AppSyncEventsUnsubscribeSuccessMessage),
    UnsubscribeError(AppSyncEventsErrorMessage),
}

/// `AppSyncEventsConnectionAckMessage` contains the connection paramters for this acknowledged
/// connection.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncEventsConnectionAckMessage {
    pub connection_timeout_ms: Option<i64>,
}

/// `AppSyncEventsSubscribeMessage` contains the parameters to subscribe to an AppSync Events channel.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncEventsSubscribeMessage {
    pub id: Option<String>,
    pub channel: Option<String>,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub authorization: HeaderMap,
}

/// `AppSyncEventsSubscribeSuccessMessage` contains the subscription parameters for this
/// successful subscription.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncEventsSubscribeSuccessMessage {
    pub id: Option<String>,
}

/// `AppSyncEventsErrorMessage` contains one or more AppSync Events errors.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncEventsErrorMessage {
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<AppSyncEventErrorDescription>>,
}

/// `AppSyncEventSubscribeErrorDescription` contains information about an error.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncEventErrorDescription {
    #[serde(default)]
    pub error_type: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

/// `AppSyncEventsDataMessage` represents an incoming event on a subscribed AppSync Events channel.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncEventsDataMessage<T1 = Value>
where
    T1: DeserializeOwned + Serialize,
{
    pub id: Option<String>,
    #[serde(
        bound = "",
        deserialize_with = "deserialize_stringified_json",
        serialize_with = "serialize_stringified_json"
    )]
    pub event: T1,
}

/// `AppSyncEventsUnsubscribeMessage` contains the parameters to unsubscribe to an AppSync Events channel.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncEventsUnsubscribeMessage {
    pub id: Option<String>,
}

/// `AppSyncEventsUnsubscribeSuccessMessage` contains the unsubscription parameters for this
/// successful unsubscription.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSyncEventsUnsubscribeSuccessMessage {
    pub id: Option<String>,
}
