use crate::{
    custom_serde::{deserialize_headers, deserialize_nullish, http_method, serialize_multi_value_headers},
    encodings::Body,
};
#[cfg(feature = "builders")]
use bon::Builder;
use http::{HeaderMap, Method};
use query_map::QueryMap;
use serde::{Deserialize, Serialize};
#[cfg(feature = "catch-all-fields")]
use serde_json::Value;

/// `VpcLatticeRequestV2` contains data coming from VPC Lattice service (V2 format)
/// see: <https://docs.aws.amazon.com/vpc-lattice/latest/ug/lambda-functions.html#receive-event-from-service>
/// for field definitions.
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcLatticeRequestV2 {
    /// The version of the event structure (always "2.0" for V2)
    #[serde(default = "default_version")]
    pub version: String,

    /// The url path for the request.
    /// Present only if the protocol is HTTP, HTTPS, or gRPC.
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// The HTTP method of the request.
    /// Present only if the protocol is HTTP, HTTPS, or gRPC.
    #[serde(deserialize_with = "http_method::deserialize_optional")]
    #[serde(serialize_with = "http_method::serialize_optional")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub method: Option<Method>,

    /// HTTP headers of the request (VPC Lattice V2 uses arrays for multi-values)
    #[serde(default, deserialize_with = "deserialize_headers")]
    #[serde(serialize_with = "serialize_multi_value_headers")]
    pub headers: HeaderMap,

    /// HTTP query string parameters (VPC Lattice V2 uses arrays for multi-values)
    #[serde(default)]
    pub query_string_parameters: QueryMap,

    /// The request body
    #[serde(default)]
    pub body: Option<Body>,

    /// Whether the body is base64 encoded
    #[serde(default, deserialize_with = "deserialize_nullish")]
    pub is_base64_encoded: bool,

    /// VPC Lattice specific request context
    pub request_context: VpcLatticeRequestV2Context,

    /// Catchall to catch any additional fields that were present but not explicitly defined by this struct.
    /// Enabled with Cargo feature `catch-all-fields`.
    /// If `catch-all-fields` is disabled, any additional fields that are present will be ignored.
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// VPC Lattice specific request context
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcLatticeRequestV2Context {
    /// ARN of the service network that delivers the request
    pub service_network_arn: String,

    /// ARN of the service that receives the request
    pub service_arn: String,

    /// ARN of the target group that receives the request
    pub target_group_arn: String,

    /// Identity information for the request.
    /// Present only if authentication is configured.
    #[serde(default)]
    pub identity: Option<VpcLatticeRequestV2Identity>,

    /// AWS region where the request is processed
    pub region: String,

    /// Time of the request in microseconds since epoch
    pub time_epoch: String,

    /// Catchall for additional context fields
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

/// Identity information in VPC Lattice request context
#[non_exhaustive]
#[cfg_attr(feature = "builders", derive(Builder))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpcLatticeRequestV2Identity {
    /// ARN of the VPC where the request originated
    #[serde(default)]
    pub source_vpc_arn: Option<String>,

    /// Type of authentication (e.g., "AWS_IAM")
    #[serde(rename = "type")]
    #[serde(default)]
    pub identity_type: Option<String>,

    /// The authenticated principal
    #[serde(default)]
    pub principal: Option<String>,

    /// Organization ID of the authenticated principal
    #[serde(rename = "principalOrgID")]
    #[serde(default)]
    pub principal_org_id: Option<String>,

    /// Name of the authenticated session
    #[serde(default)]
    pub session_name: Option<String>,

    /// X.509 certificate fields (for Roles Anywhere)
    #[serde(rename = "x509IssuerOu")]
    #[serde(default)]
    pub x509_issuer_ou: Option<String>,
    #[serde(rename = "x509SanDns")]
    #[serde(default)]
    pub x509_san_dns: Option<String>,
    #[serde(rename = "x509SanNameCn")]
    #[serde(default)]
    pub x509_san_name_cn: Option<String>,
    #[serde(rename = "x509SanUri")]
    #[serde(default)]
    pub x509_san_uri: Option<String>,
    #[serde(rename = "x509SubjectCn")]
    #[serde(default)]
    pub x509_subject_cn: Option<String>,

    /// Catchall for additional identity fields
    #[cfg(feature = "catch-all-fields")]
    #[cfg_attr(docsrs, doc(cfg(feature = "catch-all-fields")))]
    #[serde(flatten)]
    #[cfg_attr(feature = "builders", builder(default))]
    pub other: serde_json::Map<String, Value>,
}

fn default_version() -> String {
    "2.0".to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "vpc_lattice")]
    fn example_vpc_lattice_v2_deserialize() {
        let data = include_bytes!("../../fixtures/example-vpc-lattice-v2-request.json");
        let parsed: VpcLatticeRequestV2 = serde_json::from_slice(data).unwrap();

        assert_eq!(Some("/health".to_string()), parsed.path);
        assert_eq!(Some(Method::GET), parsed.method);
        assert_eq!(
            "curl/7.68.0",
            parsed.headers.get_all("user-agent").iter().next().unwrap()
        );

        // headers including testing multi-values
        let header_multi = parsed.headers.get_all("multi");
        assert_eq!("x", header_multi.iter().next().unwrap());
        assert_eq!("y", header_multi.iter().nth(1).unwrap());

        // query string including testing multi-values
        assert_eq!("prod", parsed.query_string_parameters.first("state").unwrap());
        let query_multi = parsed.query_string_parameters.all("multi").unwrap();
        assert_eq!(&"a", query_multi.first().unwrap());
        assert_eq!(&"DEF", query_multi.get(1).unwrap());
        assert_eq!(&"g", query_multi.get(2).unwrap());

        assert!(parsed.body.is_none());
        assert!(!parsed.is_base64_encoded);

        // nested structure testing
        assert_eq!(
            "arn:aws:vpc-lattice:ap-southeast-2:123456789012:service/svc-0a40eebed65f8d69c",
            parsed.request_context.service_arn
        );
        assert_eq!(
            "arn:aws:vpc-lattice:ap-southeast-2:123456789012:servicenetwork/sn-0bf3f2882e9cc805a",
            parsed.request_context.service_network_arn
        );
        assert_eq!(
            "arn:aws:vpc-lattice:ap-southeast-2:123456789012:targetgroup/tg-6d0ecf831eec9f09",
            parsed.request_context.target_group_arn
        );
        assert_eq!("ap-southeast-2", parsed.request_context.region);
        assert_eq!("1724875399456789", parsed.request_context.time_epoch);

        let context = parsed.request_context.identity.as_ref().unwrap();

        // identity
        assert_eq!(
            "arn:aws:ec2:ap-southeast-2:123456789012:vpc/vpc-0b8276c84697e7339",
            context.clone().source_vpc_arn.unwrap()
        );
        assert_eq!("AWS_IAM", context.clone().identity_type.unwrap());
        assert_eq!(
            "arn:aws:iam::123456789012:role/service-role/HealthChecker",
            context.clone().principal.unwrap()
        );
        assert_eq!("o-50dc6c495c0c9188", context.clone().principal_org_id.unwrap());
    }

    #[test]
    #[cfg(feature = "vpc_lattice")]
    fn example_vpc_lattice_v2_serde_round_trip() {
        // our basic example has instances of multi-value headers and multi-value parameters
        // so this test covers both those serialization edge cases
        let data = include_bytes!("../../fixtures/example-vpc-lattice-v2-request.json");
        let parsed: VpcLatticeRequestV2 = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: VpcLatticeRequestV2 = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "vpc_lattice")]
    fn example_vpc_lattice_v2_serde_round_trip_base64_body() {
        let data = include_bytes!("../../fixtures/example-vpc-lattice-v2-request-base64.json");
        let parsed: VpcLatticeRequestV2 = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: VpcLatticeRequestV2 = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "vpc_lattice")]
    fn example_vpc_lattice_v2_serde_round_trip_role_anywhere() {
        let data = include_bytes!("../../fixtures/example-vpc-lattice-v2-request-roles-anywhere.json");
        let parsed: VpcLatticeRequestV2 = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: VpcLatticeRequestV2 = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
