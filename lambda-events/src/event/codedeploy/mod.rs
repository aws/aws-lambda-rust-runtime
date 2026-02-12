#[cfg(feature = "builders")]
use bon::Builder;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

pub type CodeDeployDeploymentState = String;

/// `CodeDeployEvent` is documented at:
/// <https://docs.aws.amazon.com/AmazonCloudWatch/latest/events/EventTypes.html#acd_event_types>
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeDeployEvent {
    /// AccountID is the id of the AWS account from which the event originated.
    #[serde(default)]
    #[serde(rename = "account")]
    pub account_id: Option<String>,
    /// Region is the AWS region from which the event originated.
    #[serde(default)]
    pub region: Option<String>,
    /// DetailType informs the schema of the Detail field. For deployment state-change
    /// events, the value should be equal to CodeDeployDeploymentEventDetailType.
    /// For instance state-change events, the value should be equal to
    /// CodeDeployInstanceEventDetailType.
    #[serde(default)]
    #[serde(rename = "detail-type")]
    pub detail_type: Option<String>,
    /// Source should be equal to CodeDeployEventSource.
    #[serde(default)]
    pub source: Option<String>,
    /// Version is the version of the event's schema.
    #[serde(default)]
    pub version: Option<String>,
    /// Time is the event's timestamp.
    pub time: DateTime<Utc>,
    /// ID is the GUID of this event.
    #[serde(default)]
    pub id: Option<String>,
    /// Resources is a list of ARNs of CodeDeploy applications and deployment
    /// groups that this event pertains to.
    pub resources: Vec<String>,
    /// Detail contains information specific to a deployment event.
    pub detail: CodeDeployEventDetail,
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
pub struct CodeDeployEventDetail {
    /// InstanceGroupID is the ID of the instance group.
    #[serde(default)]
    pub instance_group_id: Option<String>,
    /// InstanceID is the id of the instance. This field is non-empty only if
    /// the DetailType of the complete event is CodeDeployInstanceEventDetailType.
    pub instance_id: Option<String>,
    /// Region is the AWS region that the event originated from.
    #[serde(default)]
    pub region: Option<String>,
    /// Application is the name of the CodeDeploy application.
    #[serde(default)]
    pub application: Option<String>,
    /// DeploymentID is the id of the deployment.
    #[serde(default)]
    pub deployment_id: Option<String>,
    /// State is the new state of the deployment.
    pub state: CodeDeployDeploymentState,
    /// DeploymentGroup is the name of the deployment group.
    #[serde(default)]
    pub deployment_group: Option<String>,
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
#[derive(Debug, Default, Clone, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct CodeDeployLifecycleEvent {
    pub deployment_id: String,
    pub lifecycle_event_hook_execution_id: String,
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
    #[cfg(feature = "codedeploy")]
    fn example_codedeploy_lifecycle_event() {
        let data = include_bytes!("../../fixtures/example-codedeploy-lifecycle-event.json");
        let parsed: CodeDeployLifecycleEvent = serde_json::from_slice(data).unwrap();

        assert_eq!(parsed.deployment_id, "d-deploymentId".to_string());
        assert_eq!(parsed.lifecycle_event_hook_execution_id, "eyJlbmNyeXB0ZWREYXRhIjoiY3VHU2NjdkJXUTJQUENVd2dkYUNGRVg0dWlpME9UWVdHTVhZcDRXVW5LYUVKc21EaUFPMkNLNXMwMWFrNDlYVStlbXdRb29xS3NJTUNVQ3RYRGFZSXc1VTFwUllvMDhmMzdlbDZFeDVVdjZrNFc0eU5waGh6YTRvdkNWcmVveVR6OWdERlM2SmlIYW1TZz09IiwiaXZQYXJhbWV0ZXJTcGVjIjoiTm1ZNFR6RzZxQVhHamhhLyIsIm1hdGVyaWFsU2V0U2VyaWFsIjoxfQ==".to_string());

        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CodeDeployLifecycleEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "codedeploy")]
    fn example_codedeploy_deployment_event() {
        let data = include_bytes!("../../fixtures/example-codedeploy-deployment-event.json");
        let parsed: CodeDeployEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CodeDeployEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "codedeploy")]
    fn example_codedeploy_instance_event() {
        let data = include_bytes!("../../fixtures/example-codedeploy-instance-event.json");
        let parsed: CodeDeployEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CodeDeployEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
