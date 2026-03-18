#[cfg(feature = "builders")]
use bon::Builder;
use http::{HeaderMap, Method};
use query_map::QueryMap;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

use crate::{
    custom_serde::{
        deserialize_comma_separated_headers, deserialize_nullish, http_method, serialize_comma_separated_headers,
    },
    encodings::Body,
};

/// `VpcLatticeRequestV1` contains data coming from VPC Lattice service (V1 format)
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
// we note that V1 requests are snake cased UNLIKE v2 which are camel cased
#[serde(rename_all = "snake_case")]
pub struct VpcLatticeRequestV1 {
    /// The url path for the request.
    /// Present only if the protocol is HTTP, HTTPS, or gRPC.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_path: Option<String>,

    /// The HTTP method of the request.
    /// Present only if the protocol is HTTP, HTTPS, or gRPC.
    #[serde(deserialize_with = "http_method::deserialize_optional")]
    #[serde(serialize_with = "http_method::serialize_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub method: Option<Method>,

    /// HTTP headers of the request (V1 uses comma-separated strings for multi-values)
    #[serde(deserialize_with = "deserialize_comma_separated_headers", default)]
    #[serde(serialize_with = "serialize_comma_separated_headers")]
    pub headers: HeaderMap,

    /// HTTP query string parameters (V1 uses the last value passed for multi-values
    /// so no special serializer is needed)
    #[serde(default)]
    pub query_string_parameters: QueryMap,

    /// The request body
    #[serde(default)]
    pub body: Option<Body>,

    /// Whether the body is base64 encoded
    #[serde(default, deserialize_with = "deserialize_nullish")]
    pub is_base64_encoded: bool,

    /// Catchall to catch any additional fields
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
    #[cfg(feature = "vpc_lattice")]
    fn example_vpc_lattice_v1_deserialize() {
        let data = include_bytes!("../../fixtures/example-vpc-lattice-v1-request.json");
        let parsed: VpcLatticeRequestV1 = serde_json::from_slice(data).unwrap();

        assert_eq!(Some("/api/product".to_string()), parsed.raw_path);
        assert_eq!(Some(Method::POST), parsed.method);
        assert_eq!(
            "curl/7.68.0",
            parsed.headers.get_all("user-agent").iter().next().unwrap()
        );
        assert_eq!("electronics", parsed.query_string_parameters.first("category").unwrap());
        assert_eq!(
            Body::Text("{\"id\": 5, \"description\": \"TV\"}".to_string()),
            parsed.body.unwrap()
        );
        assert!(!parsed.is_base64_encoded);
    }

    #[test]
    #[cfg(feature = "vpc_lattice")]
    fn example_vpc_lattice_v1_deserialize_headers_multi_values() {
        let data = include_bytes!("../../fixtures/example-vpc-lattice-v1-request.json");
        let parsed: VpcLatticeRequestV1 = serde_json::from_slice(data).unwrap();

        assert_eq!("abcd", parsed.headers.get_all("multi").iter().next().unwrap());
        assert_eq!("DEF", parsed.headers.get_all("multi").iter().nth(1).unwrap());
    }

    #[test]
    #[cfg(feature = "vpc_lattice")]
    fn example_vpc_lattice_v1_deserialize_query_string_map() {
        let data = include_bytes!("../../fixtures/example-vpc-lattice-v1-request.json");
        let parsed: VpcLatticeRequestV1 = serde_json::from_slice(data).unwrap();

        assert_eq!("electronics", parsed.query_string_parameters.first("category").unwrap());
        assert_eq!("tv", parsed.query_string_parameters.first("tags").unwrap());
    }
}
