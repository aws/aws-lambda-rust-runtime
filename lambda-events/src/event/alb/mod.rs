use crate::{
    custom_serde::{
        deserialize_headers, deserialize_nullish_boolean, http_method, serialize_headers,
        serialize_multi_value_headers, serialize_query_string_parameters,
    },
    encodings::Body,
};
use http::{HeaderMap, Method};
use query_map::QueryMap;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

/// `AlbTargetGroupRequest` contains data originating from the ALB Lambda target group integration
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbTargetGroupRequest {
    #[serde(with = "http_method")]
    pub http_method: Method,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    #[serde(serialize_with = "serialize_query_string_parameters")]
    pub query_string_parameters: QueryMap,
    #[serde(default)]
    pub multi_value_query_string_parameters: QueryMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "deserialize_headers", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    pub request_context: AlbTargetGroupRequestContext,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,
    pub body: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `AlbTargetGroupRequestContext` contains the information to identify the load balancer invoking the lambda
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbTargetGroupRequestContext {
    pub elb: ElbContext,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `ElbContext` contains the information to identify the ARN invoking the lambda
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElbContext {
    /// nolint: stylecheck
    #[serde(default)]
    pub target_group_arn: Option<String>,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

/// `AlbTargetGroupResponse` configures the response to be returned by the ALB Lambda target group for the request
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbTargetGroupResponse {
    pub status_code: i64,
    #[serde(default)]
    pub status_description: Option<String>,
    #[serde(deserialize_with = "http_serde::header_map::deserialize", default)]
    #[serde(serialize_with = "serialize_headers")]
    pub headers: HeaderMap,
    #[serde(deserialize_with = "http_serde::header_map::deserialize", default)]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub multi_value_headers: HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Body>,
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,
    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    pub other: serde_json::Map<String, Value>,
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::Value;

    #[test]
    #[cfg(feature = "alb")]
    fn example_alb_lambda_target_request_headers_only() {
        let data = include_bytes!("../../fixtures/example-alb-lambda-target-request-headers-only.json");
        let parsed: AlbTargetGroupRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AlbTargetGroupRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "alb")]
    fn example_alb_lambda_target_request_multivalue_headers() {
        let data = include_bytes!("../../fixtures/example-alb-lambda-target-request-multivalue-headers.json");
        let parsed: AlbTargetGroupRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AlbTargetGroupRequest = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "alb")]
    fn ensure_alb_lambda_target_request_query_string_parameter_value_is_string() {
        let data = include_bytes!("../../fixtures/example-alb-lambda-target-request-headers-only.json");
        let parsed: AlbTargetGroupRequest = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Value = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(
            reparsed["queryStringParameters"]["key"],
            Value::String("hello".to_string())
        );
    }

    #[test]
    #[cfg(feature = "alb")]
    fn example_alb_lambda_target_response() {
        let data = include_bytes!("../../fixtures/example-alb-lambda-target-response.json");
        let parsed: AlbTargetGroupResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: AlbTargetGroupResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
