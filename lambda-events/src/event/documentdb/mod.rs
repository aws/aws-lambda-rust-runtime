pub mod events;

use self::events::{
    delete_event::ChangeDeleteEvent, drop_database_event::ChangeDropDatabaseEvent, drop_event::ChangeDropEvent,
    insert_event::ChangeInsertEvent, invalidate_event::ChangeInvalidateEvent, rename_event::ChangeRenameEvent,
    replace_event::ChangeReplaceEvent, update_event::ChangeUpdateEvent,
};
#[cfg(feature = "builders")]
use bon::Builder;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "operationType", rename_all = "camelCase")]
pub enum ChangeEvent {
    Insert(ChangeInsertEvent),
    Delete(ChangeDeleteEvent),
    Drop(ChangeDropEvent),
    DropDatabase(ChangeDropDatabaseEvent),
    Invalidate(ChangeInvalidateEvent),
    Replace(ChangeReplaceEvent),
    Update(ChangeUpdateEvent),
    Rename(ChangeRenameEvent),
}

#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentDbInnerEvent {
    pub event: ChangeEvent,
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
pub struct DocumentDbEvent {
    #[serde(default)]
    pub event_source_arn: Option<String>,
    pub events: Vec<DocumentDbInnerEvent>,
    #[serde(default)]
    pub event_source: Option<String>,
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
#[cfg(feature = "documentdb")]
mod test {
    use super::*;

    pub type Event = DocumentDbEvent;

    fn test_example(data: &[u8]) {
        let parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();

        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_documentdb_insert_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-insert-event.json"));
    }

    #[test]
    fn example_documentdb_delete_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-delete-event.json"));
    }

    #[test]
    fn example_documentdb_drop_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-drop-event.json"));
    }

    #[test]
    fn example_documentdb_replace_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-replace-event.json"));
    }

    #[test]
    fn example_documentdb_update_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-update-event.json"));
    }

    #[test]
    fn example_documentdb_rename_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-rename-event.json"));
    }

    #[test]
    fn example_documentdb_invalidate_event() {
        test_example(include_bytes!(
            "../../fixtures/example-documentdb-invalidate-event.json"
        ));
    }

    #[test]
    fn example_documentdb_drop_database_event() {
        test_example(include_bytes!(
            "../../fixtures/example-documentdb-drop-database-event.json"
        ));
    }
}
