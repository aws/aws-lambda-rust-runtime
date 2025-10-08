use crate::{
    custom_serde::{codebuild_time, CodeBuildNumber},
    encodings::{MinuteDuration, SecondDuration},
};
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

pub type CodeBuildPhaseStatus = String;

pub type CodeBuildPhaseType = String;

/// `CodeBuildEvent` is documented at:
/// <https://docs.aws.amazon.com/codebuild/latest/userguide/sample-build-notifications.html#sample-build-notifications-ref>
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBuildEvent {
    /// AccountID is the id of the AWS account from which the event originated.
    #[serde(default)]
    #[serde(rename = "account")]
    pub account_id: Option<String>,
    /// Region is the AWS region from which the event originated.
    #[serde(default)]
    pub region: Option<String>,
    /// DetailType informs the schema of the Detail field. For build state-change
    /// events, the value will be CodeBuildStateChangeDetailType. For phase-change
    /// events, it will be CodeBuildPhaseChangeDetailType.
    #[serde(default)]
    #[serde(rename = "detail-type")]
    pub detail_type: Option<String>,
    /// Source should be equal to CodeBuildEventSource.
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
    /// Resources is a list of ARNs of CodeBuild builds that this event pertains to.
    pub resources: Vec<String>,
    /// Detail contains information specific to a build state-change or
    /// build phase-change event.
    pub detail: CodeBuildEventDetail,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CodeBuildEventDetail` represents the all details related to the code build event
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBuildEventDetail {
    #[serde(rename = "build-status")]
    pub build_status: Option<CodeBuildPhaseStatus>,
    #[serde(default)]
    #[serde(rename = "project-name")]
    pub project_name: Option<String>,
    #[serde(default)]
    #[serde(rename = "build-id")]
    pub build_id: Option<String>,
    #[serde(rename = "additional-information")]
    pub additional_information: CodeBuildEventAdditionalInformation,
    #[serde(rename = "current-phase")]
    pub current_phase: Option<CodeBuildPhaseType>,
    #[serde(default)]
    #[serde(rename = "current-phase-context")]
    pub current_phase_context: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(rename = "completed-phase-status")]
    pub completed_phase_status: Option<CodeBuildPhaseStatus>,
    #[serde(rename = "completed-phase")]
    pub completed_phase: Option<CodeBuildPhaseType>,
    #[serde(default)]
    #[serde(rename = "completed-phase-context")]
    pub completed_phase_context: Option<String>,
    #[serde(rename = "completed-phase-duration-seconds")]
    pub completed_phase_duration: Option<SecondDuration>,
    #[serde(rename = "completed-phase-start")]
    #[serde(default)]
    #[serde(with = "codebuild_time::optional_time")]
    pub completed_phase_start: Option<CodeBuildTime>,
    #[serde(rename = "completed-phase-end")]
    #[serde(default)]
    #[serde(with = "codebuild_time::optional_time")]
    pub completed_phase_end: Option<CodeBuildTime>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CodeBuildEventAdditionalInformation` represents additional information to the code build event
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBuildEventAdditionalInformation {
    pub artifact: CodeBuildArtifact,
    pub environment: CodeBuildEnvironment,
    #[serde(rename = "timeout-in-minutes")]
    pub timeout: MinuteDuration,
    #[serde(rename = "build-complete")]
    pub build_complete: bool,
    #[serde(rename = "build-number")]
    pub build_number: Option<CodeBuildNumber>,
    #[serde(default)]
    pub initiator: Option<String>,
    #[serde(rename = "build-start-time")]
    #[serde(with = "codebuild_time::str_time")]
    pub build_start_time: CodeBuildTime,
    pub source: CodeBuildSource,
    #[serde(default)]
    #[serde(rename = "source-version")]
    pub source_version: Option<String>,
    pub logs: CodeBuildLogs,
    pub phases: Vec<CodeBuildPhase>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CodeBuildArtifact` represents the artifact provided to build
#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBuildArtifact {
    #[serde(default)]
    #[serde(rename = "md5sum")]
    pub md5_sum: Option<String>,
    #[serde(default)]
    #[serde(rename = "sha256sum")]
    pub sha256_sum: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CodeBuildEnvironment` represents the environment for a build
#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBuildEnvironment {
    #[serde(default)]
    pub image: Option<String>,
    #[serde(rename = "privileged-mode")]
    pub privileged_mode: bool,
    #[serde(default)]
    #[serde(rename = "compute-type")]
    pub compute_type: Option<String>,
    #[serde(default)]
    pub type_: Option<String>,
    #[serde(rename = "environment-variables")]
    pub environment_variables: Vec<CodeBuildEnvironmentVariable>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CodeBuildEnvironmentVariable` encapsulate environment variables for the code build
#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBuildEnvironmentVariable {
    /// Name is the name of the environment variable.
    #[serde(default)]
    pub name: Option<String>,
    /// Type is PLAINTEXT or PARAMETER_STORE.
    #[serde(default)]
    pub type_: Option<String>,
    /// Value is the value of the environment variable.
    #[serde(default)]
    pub value: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CodeBuildSource` represent the code source will be build
#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBuildSource {
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default)]
    pub type_: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CodeBuildLogs` gives the log details of a code build
#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBuildLogs {
    #[serde(default)]
    #[serde(rename = "group-name")]
    pub group_name: Option<String>,
    #[serde(default)]
    #[serde(rename = "stream-name")]
    pub stream_name: Option<String>,
    #[serde(default)]
    #[serde(rename = "deep-link")]
    pub deep_link: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `CodeBuildPhase` represents the phase of a build and its details
#[non_exhaustive]
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBuildPhase<T1 = Value>
where
    T1: DeserializeOwned,
    T1: Serialize,
{
    #[serde(bound = "")]
    #[serde(rename = "phase-context")]
    pub phase_context: Option<Vec<T1>>,
    #[serde(rename = "start-time")]
    #[serde(with = "codebuild_time::str_time")]
    pub start_time: CodeBuildTime,
    #[serde(rename = "end-time")]
    #[serde(default)]
    #[serde(with = "codebuild_time::optional_time")]
    pub end_time: Option<CodeBuildTime>,
    #[serde(rename = "duration-in-seconds")]
    pub duration: Option<SecondDuration>,
    #[serde(rename = "phase-type")]
    pub phase_type: CodeBuildPhaseType,
    #[serde(rename = "phase-status")]
    pub phase_status: Option<CodeBuildPhaseStatus>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

pub type CodeBuildTime = DateTime<Utc>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "codebuild")]
    fn example_codebuild_phase_change() {
        let data = include_bytes!("../../fixtures/example-codebuild-phase-change.json");
        let parsed: CodeBuildEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CodeBuildEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "codebuild")]
    fn example_codebuild_state_change() {
        let data = include_bytes!("../../fixtures/example-codebuild-state-change.json");
        let parsed: CodeBuildEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: CodeBuildEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
