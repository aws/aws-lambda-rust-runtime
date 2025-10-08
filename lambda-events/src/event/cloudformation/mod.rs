use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

pub mod provider;

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "RequestType")]
pub enum CloudFormationCustomResourceRequest<P1 = Value, P2 = Value>
where
    P1: DeserializeOwned + Serialize,
    P2: DeserializeOwned + Serialize,
{
    #[serde(bound = "")]
    Create(CreateRequest<P2>),
    #[serde(bound = "")]
    Update(UpdateRequest<P1, P2>),
    #[serde(bound = "")]
    Delete(DeleteRequest<P2>),
}

impl Default for CloudFormationCustomResourceRequest {
    fn default() -> Self {
        CloudFormationCustomResourceRequest::Create(CreateRequest::default())
    }
}

#[non_exhaustive]
#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateRequest<P2 = Value>
where
    P2: DeserializeOwned + Serialize,
{
    #[serde(default)]
    pub service_token: Option<String>,
    pub request_id: String,
    #[serde(rename = "ResponseURL")]
    pub response_url: String,
    pub stack_id: String,
    pub resource_type: String,
    pub logical_resource_id: String,
    #[serde(bound = "")]
    pub resource_properties: P2,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateRequest<P1 = Value, P2 = Value>
where
    P1: DeserializeOwned + Serialize,
    P2: DeserializeOwned + Serialize,
{
    #[serde(default)]
    pub service_token: Option<String>,
    pub request_id: String,
    #[serde(rename = "ResponseURL")]
    pub response_url: String,
    pub stack_id: String,
    pub resource_type: String,
    pub logical_resource_id: String,
    pub physical_resource_id: String,
    #[serde(bound = "")]
    pub resource_properties: P2,
    #[serde(bound = "")]
    pub old_resource_properties: P1,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteRequest<P2 = Value>
where
    P2: DeserializeOwned + Serialize,
{
    #[serde(default)]
    pub service_token: Option<String>,
    pub request_id: String,
    #[serde(rename = "ResponseURL")]
    pub response_url: String,
    pub stack_id: String,
    pub resource_type: String,
    pub logical_resource_id: String,
    pub physical_resource_id: String,
    #[serde(bound = "")]
    pub resource_properties: P2,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CloudFormationCustomResourceResponse {
    pub status: CloudFormationCustomResourceResponseStatus,
    pub reason: Option<String>,
    pub physical_resource_id: String,
    pub stack_id: String,
    pub request_id: String,
    pub logical_resource_id: String,
    pub no_echo: bool,
    pub data: HashMap<String, String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[non_exhaustive]
#[derive(Clone, Default, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CloudFormationCustomResourceResponseStatus {
    #[default]
    Success,
    Failed,
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::{CloudFormationCustomResourceRequest::*, *};

    #[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct TestProperties {
        key_1: String,
        key_2: Vec<String>,
        key_3: HashMap<String, String>,
    }

    type TestRequest = CloudFormationCustomResourceRequest<TestProperties, TestProperties>;

    #[test]
    fn example_cloudformation_custom_resource_create_request() {
        let data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-create-request.json");
        let parsed: TestRequest = serde_json::from_slice(data).unwrap();

        match parsed {
            Create(_) => (),
            _ => panic!("expected Create request"),
        }

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: TestRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_cloudformation_custom_resource_update_request() {
        let data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-update-request.json");
        let parsed: TestRequest = serde_json::from_slice(data).unwrap();

        match parsed {
            Update(_) => (),
            _ => panic!("expected Update request"),
        }

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: TestRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_cloudformation_custom_resource_delete_request() {
        let data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-delete-request.json");
        let parsed: TestRequest = serde_json::from_slice(data).unwrap();

        match parsed {
            Delete(_) => (),
            _ => panic!("expected Delete request"),
        }

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: TestRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_cloudformation_custom_resource_response() {
        let data = include_bytes!("../../fixtures/example-cloudformation-custom-resource-response.json");
        let parsed: CloudFormationCustomResourceResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CloudFormationCustomResourceResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
