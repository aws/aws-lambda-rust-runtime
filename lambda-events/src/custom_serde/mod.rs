use base64::Engine;
#[cfg(feature = "appsync_events")]
use serde::{de::DeserializeOwned, ser::Error as SerError, Serialize};
use serde::{
    de::{Deserialize, Deserializer, Error as DeError},
    ser::Serializer,
};

use std::collections::HashMap;

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
    feature = "appsync_events"
))]
mod headers;
#[cfg(any(
    feature = "alb",
    feature = "apigw",
    feature = "s3",
    feature = "iot",
    feature = "lambda_function_urls",
    feature = "appsync_events"
))]
pub(crate) use self::headers::*;

#[cfg(feature = "dynamodb")]
pub(crate) mod float_unix_epoch;

#[cfg(any(feature = "alb", feature = "apigw"))]
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

/// Deserializes `HashMap<_>`, mapping JSON `null` to an empty map.
pub(crate) fn deserialize_lambda_map<'de, D, K, V>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    D: Deserializer<'de>,
    K: serde::Deserialize<'de>,
    K: std::hash::Hash,
    K: std::cmp::Eq,
    V: serde::Deserialize<'de>,
{
    // https://github.com/serde-rs/serde/issues/1098
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[cfg(feature = "dynamodb")]
/// Deserializes `Item`, mapping JSON `null` to an empty item.
pub(crate) fn deserialize_lambda_dynamodb_item<'de, D>(deserializer: D) -> Result<serde_dynamo::Item, D::Error>
where
    D: Deserializer<'de>,
{
    // https://github.com/serde-rs/serde/issues/1098
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Deserializes `HashMap<_>`, mapping JSON `null` to an empty map.
#[cfg(any(
    feature = "alb",
    feature = "apigw",
    feature = "cloudwatch_events",
    feature = "code_commit",
    feature = "cognito",
    feature = "sns",
    test
))]
pub(crate) fn deserialize_nullish_boolean<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    // https://github.com/serde-rs/serde/issues/1098
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[cfg(feature = "appsync_events")]
pub(crate) fn serialize_stringified_json<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: Serializer,
{
    let json_str = serde_json::to_string(value).map_err(S::Error::custom)?;

    json_str.serialize(serializer)
}

#[cfg(feature = "appsync_events")]
pub(crate) fn deserialize_stringified_json<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    let json_str = String::deserialize(deserializer)?;

    serde_json::from_str(&json_str).map_err(D::Error::custom)
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};

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
            #[serde(deserialize_with = "deserialize_lambda_map")]
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
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "deserialize_lambda_dynamodb_item")]
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
    }

    #[test]
    fn test_deserialize_nullish_boolean() {
        #[derive(Deserialize)]
        struct Test {
            #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
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
