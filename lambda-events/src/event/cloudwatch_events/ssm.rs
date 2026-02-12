#[cfg(feature = "builders")]
use bon::Builder;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;
use std::collections::HashMap;

#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EC2AutomationStepStatusChange {
    #[serde(rename = "ExecutionId")]
    pub execution_id: String,
    #[serde(rename = "Definition")]
    pub definition: String,
    #[serde(rename = "DefinitionVersion")]
    pub definition_version: f64,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "EndTime")]
    pub end_time: String,
    #[serde(rename = "StartTime")]
    pub start_time: String,
    #[serde(rename = "Time")]
    pub time: f64,
    #[serde(rename = "StepName")]
    pub step_name: String,
    #[serde(rename = "Action")]
    pub action: String,
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
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EC2AutomationExecutionStatusChange {
    #[serde(rename = "ExecutionId")]
    pub execution_id: String,
    #[serde(rename = "Definition")]
    pub definition: String,
    #[serde(rename = "DefinitionVersion")]
    pub definition_version: f64,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "StartTime")]
    pub start_time: String,
    #[serde(rename = "EndTime")]
    pub end_time: String,
    #[serde(rename = "Time")]
    pub time: f64,
    #[serde(rename = "ExecutedBy")]
    pub executed_by: String,
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
pub struct StateChange {
    pub state: String,
    pub at_time: String,
    pub next_transition_time: String,
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
pub struct ConfigurationComplianceStateChange {
    #[serde(rename = "last-runtime")]
    pub last_runtime: Option<String>,
    #[serde(rename = "compliance-status")]
    pub compliance_status: String,
    #[serde(rename = "resource-type")]
    pub resource_type: String,
    #[serde(rename = "resource-id")]
    pub resource_id: String,
    #[serde(rename = "compliance-type")]
    pub compliance_type: String,
    #[serde(rename = "patch-baseline-id")]
    pub patch_baseline_id: Option<String>,
    pub serverity: Option<String>,
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
pub struct MaintenanceWindowTargetRegistration {
    #[serde(rename = "window-target-id")]
    pub window_target_id: String,
    #[serde(rename = "window-id")]
    pub window_id: String,
    pub status: String,
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
pub struct MaintenanceWindowExecutionStateChange {
    #[serde(rename = "start-time")]
    pub start_time: String,
    #[serde(rename = "end-time")]
    pub end_time: String,
    #[serde(rename = "window-id")]
    pub window_id: String,
    #[serde(rename = "window-execution-id")]
    pub window_execution_id: String,
    pub status: String,
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
pub struct MaintenanceWindowTaskExecutionStateChange {
    #[serde(rename = "start-time")]
    pub start_time: String,
    #[serde(rename = "task-execution-id")]
    pub task_execution_id: String,
    #[serde(rename = "end-time")]
    pub end_time: String,
    #[serde(rename = "window-id")]
    pub window_id: String,
    #[serde(rename = "window-execution-id")]
    pub window_execution_id: String,
    pub status: String,
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
pub struct MaintenanceWindowTaskTargetInvocationStateChange {
    #[serde(rename = "start-time")]
    pub start_time: String,
    #[serde(rename = "end-time")]
    pub end_time: String,
    #[serde(rename = "window-id")]
    pub window_id: String,
    #[serde(rename = "window-execution-id")]
    pub window_execution_id: String,
    #[serde(rename = "task-execution-id")]
    pub task_execution_id: String,
    #[serde(rename = "window-target-id")]
    pub window_target_id: String,
    pub status: String,
    #[serde(rename = "owner-information")]
    pub owner_information: String,
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
pub struct MaintenanceWindowStateChange {
    #[serde(rename = "window-id")]
    pub window_id: String,
    pub status: String,
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
pub struct ParameterStoreStateChange {
    pub operation: String,
    pub name: String,
    pub r#type: String,
    pub description: String,
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
pub struct EC2CommandStatusChange {
    #[serde(rename = "command-id")]
    pub command_id: String,
    #[serde(rename = "document-name")]
    pub document_name: String,
    #[serde(rename = "expire-after")]
    pub expire_after: String,
    pub parameters: HashMap<String, String>,
    #[serde(rename = "requested-date-time")]
    pub requested_date_time: String,
    pub status: String,
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
pub struct EC2CommandInvocationStatusChange {
    #[serde(rename = "command-id")]
    pub command_id: String,
    #[serde(rename = "document-name")]
    pub document_name: String,
    #[serde(rename = "instance-id")]
    pub instance_id: String,
    #[serde(rename = "requested-date-time")]
    pub requested_date_time: String,
    pub status: String,
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
pub struct EC2StateManagerAssociationStateChange {
    #[serde(rename = "association-id")]
    pub association_id: String,
    #[serde(rename = "document-name")]
    pub document_name: String,
    #[serde(rename = "association-version")]
    pub association_version: String,
    #[serde(rename = "document-version")]
    pub document_version: String,
    pub targets: String,
    #[serde(rename = "creation-date")]
    pub creation_date: String,
    #[serde(rename = "last-successful-execution-date")]
    pub last_successful_execution_date: String,
    #[serde(rename = "last-execution-date")]
    pub last_execution_date: String,
    #[serde(rename = "last-updated-date")]
    pub last_updated_date: String,
    pub status: String,
    #[serde(rename = "association-status-aggregated-count")]
    pub association_status_aggregated_count: String,
    #[serde(rename = "schedule-expression")]
    pub schedule_expression: String,
    #[serde(rename = "association-cwe-version")]
    pub association_cwe_version: String,
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
pub struct EC2StateManagerInstanceAssociationStateChange {
    #[serde(rename = "association-id")]
    pub association_id: String,
    #[serde(rename = "instance-id")]
    pub instance_id: String,
    #[serde(rename = "document-name")]
    pub document_name: String,
    #[serde(rename = "document-version")]
    pub document_version: String,
    pub targets: String,
    #[serde(rename = "creation-date")]
    pub creation_date: String,
    #[serde(rename = "last-successful-execution-date")]
    pub last_successful_execution_date: String,
    #[serde(rename = "last-execution-date")]
    pub last_execution_date: String,
    pub status: String,
    #[serde(rename = "detailed-status")]
    pub detailed_status: String,
    #[serde(rename = "error-code")]
    pub error_code: String,
    #[serde(rename = "execution-summary")]
    pub execution_summary: String,
    #[serde(rename = "output-url")]
    pub output_url: String,
    #[serde(rename = "instance-association-cwe-version")]
    pub instance_association_cwe_version: String,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}
