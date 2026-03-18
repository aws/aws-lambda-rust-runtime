use base64::Engine;
use serde::{
    de::{Deserialize, Deserializer, Error as DeError},
    ser::Serializer,
};

#[cfg(feature = "codebuild")]
pub(crate) mod codebuild_time;
#[cfg(feature = "codebuild")]
#[cfg_attr(docsrs, doc(cfg(feature = "codebuild")))]
pub type CodeBuildNumber = f32;

#[cfg(any(
    feature = "alb",
    feature = "apigw",
    feature = "s3",
    feature = "iot",
    feature = "lambda_function_urls",
    feature = "vpc_lattice"
))]
mod headers;
#[cfg(any(
    feature = "alb",
    feature = "apigw",
    feature = "s3",
    feature = "iot",
    feature = "lambda_function_urls",
    feature = "vpc_lattice"
))]
pub(crate) use self::headers::*;

#[cfg(feature = "dynamodb")]
pub(crate) mod float_unix_epoch;

#[cfg(any(feature = "alb", feature = "apigw", feature = "vpc_lattice"))]
pub(crate) mod http_method;

#[cfg(feature = "alb")]
mod query_string_parameters;
#[cfg(feature = "alb")]
pub(crate) use self::query_string_parameters::*;

pub(crate) fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(DeError::custom)
}

pub(crate) fn serialize_base64<S>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::engine::general_purpose::STANDARD.encode(value))
}

/// Deserializes any `Default` type, mapping JSON `null` to `T::default()`.
///
/// **Note** null-to-empty semantics are usually clear for container types (Map, Vec, etc).
/// For most other data types, prefer modeling fields as ```Option<T>``` with #[serde(default)]
/// instead of using this deserializer. Option preserves information about the message
/// for the application, and default semantics for the target data type may change
/// over time without warning.
pub(crate) fn deserialize_nullish<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use super::*;

    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[test]
    fn test_deserialize_base64() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_base64")]
            v: Vec<u8>,
        }
        let data = serde_json::json!({
            "v": "SGVsbG8gV29ybGQ=",
        });
        let decoded: Test = serde_json::from_value(data).unwrap();
        assert_eq!(String::from_utf8(decoded.v).unwrap(), "Hello World".to_string());
    }

    #[test]
    fn test_serialize_base64() {
        #[derive(Serialize)]
        struct Test {
            #[serde(serialize_with = "serialize_base64")]
            v: Vec<u8>,
        }
        let instance = Test {
            v: "Hello World".as_bytes().to_vec(),
        };
        let encoded = serde_json::to_string(&instance).unwrap();
        assert_eq!(encoded, r#"{"v":"SGVsbG8gV29ybGQ="}"#.to_string());
    }

    #[test]
    fn test_deserialize_map() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_nullish")]
            v: HashMap<String, String>,
        }
        let input = serde_json::json!({
          "v": {},
        });
        let decoded: Test = serde_json::from_value(input).unwrap();
        assert_eq!(HashMap::new(), decoded.v);

        let input = serde_json::json!({
          "v": null,
        });
        let decoded: Test = serde_json::from_value(input).unwrap();
        assert_eq!(HashMap::new(), decoded.v);
    }

    #[cfg(feature = "dynamodb")]
    #[test]
    fn test_deserialize_lambda_dynamodb_item() {
        #[derive(Deserialize, Debug)]
        struct Test {
            #[serde(deserialize_with = "deserialize_nullish")]
            v: serde_dynamo::Item,
        }
        let input = serde_json::json!({
          "v": {},
        });
        let decoded: Test = serde_json::from_value(input).unwrap();
        assert_eq!(serde_dynamo::Item::from(HashMap::new()), decoded.v);

        let input = serde_json::json!({
          "v": null,
        });
        let decoded: Test = serde_json::from_value(input).unwrap();
        assert_eq!(serde_dynamo::Item::from(HashMap::new()), decoded.v);

        let input = serde_json::json!({});
        let failure = serde_json::from_value::<Test>(input);
        assert!(failure.is_err(), "Missing field should not default: {failure:?}")
    }

    #[test]
    fn test_deserialize_nullish() {
        #[derive(Debug, Default, Deserialize, PartialEq)]
        struct Inner {
            x: u32,
        }
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "deserialize_nullish")]
            v: Inner,
        }

        let decoded: Test = serde_json::from_str(r#"{"v": null}"#).unwrap();
        assert_eq!(decoded.v, Inner::default());

        let decoded: Test = serde_json::from_str(r#"{}"#).unwrap();
        assert_eq!(decoded.v, Inner::default());

        let decoded: Test = serde_json::from_str(r#"{"v": {"x": 42}}"#).unwrap();
        assert_eq!(decoded.v, Inner { x: 42 });
    }

    #[test]
    fn test_deserialize_nullish_boolean() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "deserialize_nullish")]
            v: bool,
        }

        let test = r#"{"v": null}"#;
        let decoded: Test = serde_json::from_str(test).unwrap();
        assert!(!decoded.v);

        let test = r#"{}"#;
        let decoded: Test = serde_json::from_str(test).unwrap();
        assert!(!decoded.v);

        let test = r#"{"v": true}"#;
        let decoded: Test = serde_json::from_str(test).unwrap();
        assert!(decoded.v);

        let test = r#"{"v": false}"#;
        let decoded: Test = serde_json::from_str(test).unwrap();
        assert!(!decoded.v);
    }
}
