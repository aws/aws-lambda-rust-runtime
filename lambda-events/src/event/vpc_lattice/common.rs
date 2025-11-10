use crate::custom_serde::{deserialize_headers, serialize_headers};
use crate::encodings::Body;
use http::HeaderMap;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

/// `VpcLatticeResponse` configures the response to be returned
/// by VPC Lattice (both V1 and V2) for the request
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcLatticeResponse {
    // https://docs.aws.amazon.com/vpc-lattice/latest/ug/lambda-functions.html#respond-to-service
    /// Whether the body is base64 encoded
    #[serde(default)]
    pub is_base64_encoded: bool,

    /// The HTTP status code for the request
    pub status_code: u16,

    /// The HTTP status description (optional)
    #[serde(default)]
    pub status_description: Option<String>,

    /// The Http headers to return
    #[serde(deserialize_with = "deserialize_headers")]
    #[serde(serialize_with = "serialize_headers")]
    #[serde(skip_serializing_if = "HeaderMap::is_empty")]
    #[serde(default)]
    pub headers: HeaderMap,

    /// The response body
    #[serde(default)]
    pub body: Option<Body>,

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

    #[test]
    #[cfg(feature = "vpc_lattice")]
    fn example_alb_lambda_target_response() {
        let data = include_bytes!("../../fixtures/example-vpc-lattice-response.json");
        let parsed: VpcLatticeResponse = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: VpcLatticeResponse = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
