use http::{HeaderMap, Method};
use query_map::QueryMap;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

use crate::{
    custom_serde::{deserialize_nullish_boolean, http_method},
    vpc_lattice::{deserialize_comma_separated_headers, serialize_comma_separated_headers},
};

/// `VpcLatticeRequestV1` contains data coming from VPC Lattice service (V1 format)
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
// we note that V1 requests are snake cased UNLIKE v2 which are camel cased
#[serde(rename_all = "snake_case")]
pub struct VpcLatticeRequestV1 {
    /// The url path for the request
    #[serde(default)]
    pub raw_path: Option<String>,

    /// The HTTP method of the request
    #[serde(with = "http_method")]
    pub method: Method,

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
    pub body: Option<String>,

    /// Whether the body is base64 encoded
    #[serde(default, deserialize_with = "deserialize_nullish_boolean")]
    pub is_base64_encoded: bool,

    /// Catchall to catch any additional fields
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
    fn example_vpc_lattice_v1_deserialize() {
        let data = include_bytes!("../../fixtures/example-vpc-lattice-v1-request.json");
        let parsed: VpcLatticeRequestV1 = serde_json::from_slice(data).unwrap();

        assert_eq!("/api/product", parsed.raw_path.unwrap());
        assert_eq!("POST", parsed.method);
        assert_eq!(
            "curl/7.68.0",
            parsed.headers.get_all("user-agent").iter().next().unwrap()
        );
        assert_eq!("electronics", parsed.query_string_parameters.first("category").unwrap());
        assert_eq!("{\"id\": 5, \"description\": \"TV\"}", parsed.body.unwrap());
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
