use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};

/// verify that the given UTF8 JSON data deserializes to an event,
/// and that the serialized representation is an equivalent event.
///
/// ## Returns
/// The deserialized event, for further verification if needed.
pub(crate) fn verify_serde_roundtrip<T>(json_text: &[u8]) -> T
where
    T: DeserializeOwned + Serialize + PartialEq + Debug,
{
    let parsed: T = serde_json::from_slice(json_text).unwrap();
    let output: String = serde_json::to_string(&parsed).unwrap();
    let reparsed: T = serde_json::from_slice(output.as_bytes()).unwrap();
    assert_eq!(parsed, reparsed);
    parsed
}
