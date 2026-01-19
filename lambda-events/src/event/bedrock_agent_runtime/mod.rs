#[cfg(feature = "builders")]
use bon::Builder;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;
use std::collections::HashMap;

/// The Event sent to Lambda from Agents for Amazon Bedrock.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentEvent {
    ///The version of the message that identifies the format of the event data going into the Lambda function and the expected format of the response from a Lambda function. Amazon Bedrock only supports version 1.0.
    pub message_version: String,
    ///Contains information about the name, ID, alias, and version of the agent that the action group belongs to.
    pub agent: Agent,
    ///The user input for the conversation turn.
    pub input_text: String,
    /// The unique identifier of the agent session.
    pub session_id: String,
    /// The name of the action group.
    pub action_group: String,
    /// The path to the API operation, as defined in the OpenAPI schema.
    pub api_path: String,
    /// The method of the API operation, as defined in the OpenAPI schema.
    pub http_method: String,
    /// Contains a list of objects. Each object contains the name, type, and value of a parameter in the API operation, as defined in the OpenAPI schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
    /// Contains the request body and its properties, as defined in the OpenAPI schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    /// Contains session attributes and their values.
    pub session_attributes: HashMap<String, String>,
    /// Contains prompt attributes and their values.
    pub prompt_session_attributes: HashMap<String, String>,
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
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    /// Contains the request body and its properties
    pub content: HashMap<String, Content>,
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
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    /// The content of the request body
    pub properties: Vec<Property>,
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
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    /// The name of the parameter
    pub name: String,
    /// The type of the parameter
    pub r#type: String,
    /// The value of the parameter
    pub value: String,
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
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    /// The name of the parameter
    pub name: String,
    /// The type of the parameter
    pub r#type: String,
    /// The value of the parameter
    pub value: String,
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
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    /// The name of the agent.
    pub name: String,
    /// The unique identifier of the agent.
    pub id: String,
    /// The alias of the agent.
    pub alias: String,
    /// The version of the agent.
    pub version: String,
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
mod tests {

    use crate::event::bedrock_agent_runtime::AgentEvent;

    #[test]
    #[cfg(feature = "bedrock_agent_runtime")]
    fn example_bedrock_agent_runtime_event() {
        let data = include_bytes!("../../fixtures/example-bedrock-agent-runtime-event.json");
        let parsed: AgentEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AgentEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
    #[test]
    #[cfg(feature = "bedrock_agent_runtime")]
    fn example_bedrock_agent_runtime_event_without_parameters() {
        let data = include_bytes!("../../fixtures/example-bedrock-agent-runtime-event-without-parameters.json");
        let parsed: AgentEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AgentEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
    #[test]
    #[cfg(feature = "bedrock_agent_runtime")]
    fn example_bedrock_agent_runtime_event_without_request_body() {
        let data = include_bytes!("../../fixtures/example-bedrock-agent-runtime-event-without-request-body.json");
        let parsed: AgentEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AgentEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
