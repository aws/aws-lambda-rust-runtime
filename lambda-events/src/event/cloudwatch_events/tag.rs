#[cfg(feature = "catch-all-fields")]
use serde_json::Value;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagChangeOnResource {
    #[serde(rename = "changed-tag-keys")]
    pub changed_tag_keys: Vec<String>,
    pub service: String,
    #[serde(rename = "resource-type")]
    pub resource_type: String,
    pub version: i64,
    pub tags: HashMap<String, String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}
