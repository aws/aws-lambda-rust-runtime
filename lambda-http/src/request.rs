//! ALB and API Gateway request adaptations
//!
//! Typically these are exposed via the [`request_context()`] or [`request_context_ref()`]
//! request extension methods provided by the [`RequestExt`] trait.
//!
//! [`request_context()`]: crate::RequestExt::request_context()
//! [`request_context_ref()`]: crate::RequestExt::request_context_ref()
//! [`RequestExt`]: crate::RequestExt
#[cfg(any(feature = "apigw_rest", feature = "apigw_http", feature = "apigw_websockets"))]
use crate::ext::extensions::{PathParameters, StageVariables};
#[cfg(any(
    feature = "apigw_rest",
    feature = "apigw_http",
    feature = "alb",
    feature = "apigw_websockets"
))]
use crate::ext::extensions::{QueryStringParameters, RawHttpPath};
#[cfg(feature = "alb")]
use aws_lambda_events::alb::{AlbTargetGroupRequest, AlbTargetGroupRequestContext};
#[cfg(any(feature = "apigw_rest", feature = "apigw_http", feature = "apigw_websockets"))]
use aws_lambda_events::apigw::ApiGatewayRequestAuthorizer;
#[cfg(feature = "apigw_rest")]
use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyRequestContext};
#[cfg(feature = "apigw_http")]
use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpRequestContext};
#[cfg(feature = "apigw_websockets")]
use aws_lambda_events::apigw::{ApiGatewayWebsocketProxyRequest, ApiGatewayWebsocketProxyRequestContext};
use aws_lambda_events::{encodings::Body, query_map::QueryMap};
use http::{header::HeaderName, HeaderMap, HeaderValue};

use serde::{Deserialize, Serialize};
use serde_json::error::Error as JsonError;

use std::{env, future::Future, io::Read, pin::Pin};
use url::Url;

/// Internal representation of an Lambda http event from
/// ALB, API Gateway REST and HTTP API proxy event perspectives
///
/// This is not intended to be a type consumed by crate users directly. The order
/// of the variants are notable. Serde will try to deserialize in this order.
#[non_exhaustive]
#[doc(hidden)]
#[derive(Debug)]
pub enum LambdaRequest {
    #[cfg(feature = "apigw_rest")]
    ApiGatewayV1(ApiGatewayProxyRequest),
    #[cfg(feature = "apigw_http")]
    ApiGatewayV2(ApiGatewayV2httpRequest),
    #[cfg(feature = "alb")]
    Alb(AlbTargetGroupRequest),
    #[cfg(feature = "apigw_websockets")]
    WebSocket(ApiGatewayWebsocketProxyRequest),
    #[cfg(feature = "pass_through")]
    PassThrough(String),
}

impl LambdaRequest {
    /// Return the `RequestOrigin` of the request to determine where the `LambdaRequest`
    /// originated from, so that the appropriate response can be selected based on what
    /// type of response the request origin expects.
    pub fn request_origin(&self) -> RequestOrigin {
        match self {
            #[cfg(feature = "apigw_rest")]
            LambdaRequest::ApiGatewayV1 { .. } => RequestOrigin::ApiGatewayV1,
            #[cfg(feature = "apigw_http")]
            LambdaRequest::ApiGatewayV2 { .. } => RequestOrigin::ApiGatewayV2,
            #[cfg(feature = "alb")]
            LambdaRequest::Alb { .. } => RequestOrigin::Alb,
            #[cfg(feature = "apigw_websockets")]
            LambdaRequest::WebSocket { .. } => RequestOrigin::WebSocket,
            #[cfg(feature = "pass_through")]
            LambdaRequest::PassThrough { .. } => RequestOrigin::PassThrough,
            #[cfg(not(any(
                feature = "apigw_rest",
                feature = "apigw_http",
                feature = "alb",
                feature = "apigw_websockets"
            )))]
            _ => compile_error!("Either feature `apigw_rest`, `apigw_http`, `alb`, or `apigw_websockets` must be enabled for the `lambda-http` crate."),
        }
    }
}

/// RequestFuture type
pub type RequestFuture<'a, R, E> = Pin<Box<dyn Future<Output = Result<R, E>> + Send + 'a>>;

/// Represents the origin from which the lambda was requested from.
#[non_exhaustive]
#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum RequestOrigin {
    /// API Gateway request origin
    #[cfg(feature = "apigw_rest")]
    ApiGatewayV1,
    /// API Gateway v2 request origin
    #[cfg(feature = "apigw_http")]
    ApiGatewayV2,
    /// ALB request origin
    #[cfg(feature = "alb")]
    Alb,
    /// API Gateway WebSocket
    #[cfg(feature = "apigw_websockets")]
    WebSocket,
    /// PassThrough request origin
    #[cfg(feature = "pass_through")]
    PassThrough,
}

#[cfg(feature = "apigw_http")]
fn into_api_gateway_v2_request(ag: ApiGatewayV2httpRequest) -> http::Request<Body> {
    let http_method = ag.request_context.http.method.clone();
    let host = ag
        .headers
        .get(http::header::HOST)
        .and_then(|s| s.to_str().ok())
        .or(ag.request_context.domain_name.as_deref());
    let raw_path = ag.raw_path.unwrap_or_default();
    let path = apigw_path_with_stage(&ag.request_context.stage, &raw_path);

    // don't use the query_string_parameters from API GW v2 to
    // populate the QueryStringParameters extension because
    // the value is not compatible with the whatgw specification.
    // See: https://github.com/awslabs/aws-lambda-rust-runtime/issues/470
    // See: https://url.spec.whatwg.org/#urlencoded-parsing
    let query_string_parameters = if let Some(query) = &ag.raw_query_string {
        query.parse().unwrap() // this is Infallible
    } else {
        ag.query_string_parameters
    };

    let mut uri = build_request_uri(&path, &ag.headers, host, None);
    if let Some(query) = ag.raw_query_string {
        if !query.is_empty() {
            uri.push('?');
            uri.push_str(&query);
        }
    }

    let builder = http::Request::builder()
        .uri(uri)
        .extension(RawHttpPath(raw_path))
        .extension(QueryStringParameters(query_string_parameters))
        .extension(PathParameters(QueryMap::from(ag.path_parameters)))
        .extension(StageVariables(QueryMap::from(ag.stage_variables)))
        .extension(RequestContext::ApiGatewayV2(ag.request_context));

    let mut headers = ag.headers;
    update_xray_trace_id_header(&mut headers);
    if let Some(cookies) = ag.cookies {
        if let Ok(header_value) = HeaderValue::from_str(&cookies.join(";")) {
            headers.insert(http::header::COOKIE, header_value);
        }
    }

    let base64 = ag.is_base64_encoded;

    let mut req = builder
        .body(
            ag.body
                .as_deref()
                .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
        )
        .expect("failed to build request");

    // no builder method that sets headers in batch
    let _ = std::mem::replace(req.headers_mut(), headers);
    let _ = std::mem::replace(req.method_mut(), http_method);

    req
}

fn update_xray_trace_id_header(headers: &mut HeaderMap) {
    if let Ok(xray_trace_id) = env::var("_X_AMZN_TRACE_ID") {
        if let Ok(header_value) = HeaderValue::from_str(&xray_trace_id) {
            headers.insert(HeaderName::from_static("x-amzn-trace-id"), header_value);
        }
    }
}
#[cfg(feature = "apigw_rest")]
fn into_proxy_request(ag: ApiGatewayProxyRequest) -> http::Request<Body> {
    let http_method = ag.http_method;
    let host = ag
        .headers
        .get(http::header::HOST)
        .and_then(|s| s.to_str().ok())
        .or(ag.request_context.domain_name.as_deref());
    let raw_path = ag.path.unwrap_or_default();
    let path = apigw_path_with_stage(&ag.request_context.stage, &raw_path);

    let builder = http::Request::builder()
        .uri(build_request_uri(
            &path,
            &ag.headers,
            host,
            Some((&ag.multi_value_query_string_parameters, &ag.query_string_parameters)),
        ))
        .extension(RawHttpPath(raw_path))
        // multi-valued query string parameters are always a super
        // set of singly valued query string parameters,
        // when present, multi-valued query string parameters are preferred
        .extension(QueryStringParameters(
            if ag.multi_value_query_string_parameters.is_empty() {
                ag.query_string_parameters
            } else {
                ag.multi_value_query_string_parameters
            },
        ))
        .extension(PathParameters(QueryMap::from(ag.path_parameters)))
        .extension(StageVariables(QueryMap::from(ag.stage_variables)))
        .extension(RequestContext::ApiGatewayV1(ag.request_context));

    // merge headers into multi_value_headers and make
    // multi-value_headers our cannoncial source of request headers
    let mut headers = ag.multi_value_headers;
    headers.extend(ag.headers);
    update_xray_trace_id_header(&mut headers);

    let base64 = ag.is_base64_encoded;
    let mut req = builder
        .body(
            ag.body
                .as_deref()
                .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
        )
        .expect("failed to build request");

    // no builder method that sets headers in batch
    let _ = std::mem::replace(req.headers_mut(), headers);
    let _ = std::mem::replace(req.method_mut(), http_method);

    req
}

#[cfg(feature = "alb")]
fn into_alb_request(alb: AlbTargetGroupRequest) -> http::Request<Body> {
    let http_method = alb.http_method;
    let host = alb.headers.get(http::header::HOST).and_then(|s| s.to_str().ok());
    let raw_path = alb.path.unwrap_or_default();

    let query_string_parameters = decode_query_map(alb.query_string_parameters);
    let multi_value_query_string_parameters = decode_query_map(alb.multi_value_query_string_parameters);

    let builder = http::Request::builder()
        .uri(build_request_uri(
            &raw_path,
            &alb.headers,
            host,
            Some((&multi_value_query_string_parameters, &query_string_parameters)),
        ))
        .extension(RawHttpPath(raw_path))
        // multi valued query string parameters are always a super
        // set of singly valued query string parameters,
        // when present, multi-valued query string parameters are preferred
        .extension(QueryStringParameters(
            if multi_value_query_string_parameters.is_empty() {
                query_string_parameters
            } else {
                multi_value_query_string_parameters
            },
        ))
        .extension(RequestContext::Alb(alb.request_context));

    // merge headers into multi_value_headers and make
    // multi-value_headers our cannoncial source of request headers
    let mut headers = alb.multi_value_headers;
    headers.extend(alb.headers);
    update_xray_trace_id_header(&mut headers);

    let base64 = alb.is_base64_encoded;

    let mut req = builder
        .body(
            alb.body
                .as_deref()
                .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
        )
        .expect("failed to build request");

    // no builder method that sets headers in batch
    let _ = std::mem::replace(req.headers_mut(), headers);
    let _ = std::mem::replace(req.method_mut(), http_method);

    req
}

#[cfg(feature = "alb")]
fn decode_query_map(query_map: QueryMap) -> QueryMap {
    use std::str::FromStr;

    let query_string = query_map.to_query_string();
    let decoded = percent_encoding::percent_decode(query_string.as_bytes()).decode_utf8_lossy();
    QueryMap::from_str(&decoded).unwrap_or_default()
}

#[cfg(feature = "apigw_websockets")]
fn into_websocket_request(ag: ApiGatewayWebsocketProxyRequest) -> http::Request<Body> {
    let http_method = ag.http_method;
    let host = ag
        .headers
        .get(http::header::HOST)
        .and_then(|s| s.to_str().ok())
        .or(ag.request_context.domain_name.as_deref());
    let raw_path = ag.path.unwrap_or_default();
    let path = apigw_path_with_stage(&ag.request_context.stage, &raw_path);

    let builder = http::Request::builder()
        .uri(build_request_uri(
            &path,
            &ag.headers,
            host,
            Some((&ag.multi_value_query_string_parameters, &ag.query_string_parameters)),
        ))
        .extension(RawHttpPath(raw_path))
        // multi-valued query string parameters are always a super
        // set of singly valued query string parameters,
        // when present, multi-valued query string parameters are preferred
        .extension(QueryStringParameters(
            if ag.multi_value_query_string_parameters.is_empty() {
                ag.query_string_parameters
            } else {
                ag.multi_value_query_string_parameters
            },
        ))
        .extension(PathParameters(QueryMap::from(ag.path_parameters)))
        .extension(StageVariables(QueryMap::from(ag.stage_variables)))
        .extension(RequestContext::WebSocket(ag.request_context));

    // merge headers into multi_value_headers and make
    // multi-value_headers our canonical source of request headers
    let mut headers = ag.multi_value_headers;
    headers.extend(ag.headers);
    update_xray_trace_id_header(&mut headers);

    let base64 = ag.is_base64_encoded;
    let mut req = builder
        .body(
            ag.body
                .as_deref()
                .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
        )
        .expect("failed to build request");

    // no builder method that sets headers in batch
    let _ = std::mem::replace(req.headers_mut(), headers);
    let _ = std::mem::replace(req.method_mut(), http_method.unwrap_or(http::Method::GET));

    req
}

#[cfg(feature = "pass_through")]
fn into_pass_through_request(data: String) -> http::Request<Body> {
    let mut builder = http::Request::builder();

    let headers = builder.headers_mut().unwrap();
    headers.insert("Content-Type", "application/json".parse().unwrap());

    update_xray_trace_id_header(headers);

    let raw_path = "/events";

    builder
        .method(http::Method::POST)
        .uri(raw_path)
        .extension(RawHttpPath(raw_path.to_string()))
        .extension(RequestContext::PassThrough)
        .body(Body::from(data))
        .expect("failed to build request")
}

#[cfg(any(feature = "apigw_rest", feature = "apigw_http", feature = "apigw_websockets"))]
fn apigw_path_with_stage(stage: &Option<String>, path: &str) -> String {
    if env::var("AWS_LAMBDA_HTTP_IGNORE_STAGE_IN_PATH").is_ok() {
        return path.into();
    }

    let stage = match stage {
        None => return path.into(),
        Some(stage) if stage == "$default" => return path.into(),
        Some(stage) => stage,
    };

    let prefix = format!("/{stage}/");
    if path.starts_with(&prefix) {
        path.into()
    } else {
        format!("/{stage}{path}")
    }
}

/// Event request context as an enumeration of request contexts
/// for both ALB and API Gateway and HTTP API events
#[non_exhaustive]
#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum RequestContext {
    /// API Gateway proxy request context
    #[cfg(feature = "apigw_rest")]
    ApiGatewayV1(ApiGatewayProxyRequestContext),
    /// API Gateway v2 request context
    #[cfg(feature = "apigw_http")]
    ApiGatewayV2(ApiGatewayV2httpRequestContext),
    /// ALB request context
    #[cfg(feature = "alb")]
    Alb(AlbTargetGroupRequestContext),
    /// WebSocket request context
    #[cfg(feature = "apigw_websockets")]
    WebSocket(ApiGatewayWebsocketProxyRequestContext),
    /// Custom request context
    #[cfg(feature = "pass_through")]
    PassThrough,
}

/// Converts LambdaRequest types into `http::Request<Body>` types
impl From<LambdaRequest> for http::Request<Body> {
    fn from(value: LambdaRequest) -> Self {
        match value {
            #[cfg(feature = "apigw_rest")]
            LambdaRequest::ApiGatewayV1(ag) => into_proxy_request(ag),
            #[cfg(feature = "apigw_http")]
            LambdaRequest::ApiGatewayV2(ag) => into_api_gateway_v2_request(ag),
            #[cfg(feature = "alb")]
            LambdaRequest::Alb(alb) => into_alb_request(alb),
            #[cfg(feature = "apigw_websockets")]
            LambdaRequest::WebSocket(ag) => into_websocket_request(ag),
            #[cfg(feature = "pass_through")]
            LambdaRequest::PassThrough(data) => into_pass_through_request(data),
        }
    }
}

impl RequestContext {
    /// Returns the Api Gateway Authorizer information for a request.
    #[cfg(any(feature = "apigw_rest", feature = "apigw_http", feature = "apigw_websockets"))]
    pub fn authorizer(&self) -> Option<&ApiGatewayRequestAuthorizer> {
        match self {
            #[cfg(feature = "apigw_rest")]
            Self::ApiGatewayV1(ag) => Some(&ag.authorizer),
            #[cfg(feature = "apigw_http")]
            Self::ApiGatewayV2(ag) => ag.authorizer.as_ref(),
            #[cfg(feature = "apigw_websockets")]
            Self::WebSocket(ag) => Some(&ag.authorizer),
            #[cfg(any(feature = "alb", feature = "pass_through"))]
            _ => None,
        }
    }
}

/// Deserializes a `Request` from a `Read` impl providing JSON events.
///
/// # Example
///
/// ```rust,no_run
/// use lambda_http::request::from_reader;
/// use std::fs::File;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let request = from_reader(
///         File::open("path/to/request.json")?
///     )?;
///     Ok(println!("{:#?}", request))
/// }
/// ```
pub fn from_reader<R>(rdr: R) -> Result<crate::Request, JsonError>
where
    R: Read,
{
    serde_json::from_reader(rdr).map(LambdaRequest::into)
}

/// Deserializes a `Request` from a string of JSON text.
///
/// # Example
///
/// ```rust,no_run
/// use lambda_http::request::from_str;
/// use std::fs::File;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let request = from_str(
///         r#"{ ...raw json here... }"#
///     )?;
///     Ok(println!("{:#?}", request))
/// }
/// ```
pub fn from_str(s: &str) -> Result<crate::Request, JsonError> {
    serde_json::from_str(s).map(LambdaRequest::into)
}

fn x_forwarded_proto() -> HeaderName {
    HeaderName::from_static("x-forwarded-proto")
}

fn build_request_uri(
    path: &str,
    headers: &HeaderMap,
    host: Option<&str>,
    queries: Option<(&QueryMap, &QueryMap)>,
) -> String {
    let mut url = match host {
        None => {
            let rel_url = Url::parse(&format!("http://localhost{path}")).unwrap();
            rel_url.path().to_string()
        }
        Some(host) => {
            let scheme = headers
                .get(x_forwarded_proto())
                .and_then(|s| s.to_str().ok())
                .unwrap_or("https");
            let url = format!("{scheme}://{host}{path}");
            Url::parse(&url).unwrap().to_string()
        }
    };

    if let Some((mv, sv)) = queries {
        if !mv.is_empty() {
            url.push('?');
            url.push_str(&mv.to_query_string());
        } else if !sv.is_empty() {
            url.push('?');
            url.push_str(&sv.to_query_string());
        }
    }

    url
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ext::RequestExt;
    use std::fs::File;

    #[test]
    fn deserializes_apigw_request_events_from_readables() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        // note: file paths are relative to the directory of the crate at runtime
        let result = from_reader(File::open("tests/data/apigw_proxy_request.json").expect("expected file"));
        assert!(result.is_ok(), "event was not parsed as expected {result:?}");
    }

    #[test]
    fn deserializes_minimal_apigw_http_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        let input = include_str!("../tests/data/apigw_v2_proxy_request_minimal.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(req.uri(), "https://xxx.execute-api.us-east-1.amazonaws.com/");

        // Ensure this is an APIGWv2 request
        let req_context = req.request_context_ref().expect("Request is missing RequestContext");
        assert!(
            matches!(req_context, &RequestContext::ApiGatewayV2(_)),
            "expected ApiGatewayV2 context, got {req_context:?}"
        );
    }

    #[test]
    fn deserializes_apigw_http_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/eventsources.html#eventsources-api-gateway-request
        let input = include_str!("../tests/data/apigw_v2_proxy_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");
        let cookie_header = req
            .headers()
            .get(http::header::COOKIE)
            .ok_or_else(|| "Cookie header not found".to_string())
            .and_then(|v| v.to_str().map_err(|e| e.to_string()));

        assert_eq!(req.method(), "POST");
        assert_eq!(req.uri(), "https://id.execute-api.us-east-1.amazonaws.com/my/path?parameter1=value1&parameter1=value2&parameter2=value");
        assert_eq!(cookie_header, Ok("cookie1=value1;cookie2=value2"));

        // Ensure this is an APIGWv2 request
        let req_context = req.request_context_ref().expect("Request is missing RequestContext");
        assert!(
            matches!(req_context, &RequestContext::ApiGatewayV2(_)),
            "expected ApiGatewayV2 context, got {req_context:?}"
        );

        let (parts, _) = req.into_parts();
        assert_eq!("https://id.execute-api.us-east-1.amazonaws.com/my/path?parameter1=value1&parameter1=value2&parameter2=value", parts.uri.to_string());
    }

    #[test]
    fn deserializes_apigw_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/http-api-develop-integrations-lambda.html
        let input = include_str!("../tests/data/apigw_proxy_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(
            req.uri(),
            "https://wt6mne2s9k.execute-api.us-west-2.amazonaws.com/test/hello?name=me"
        );

        // Ensure this is an APIGW request
        let req_context = req.request_context_ref().expect("Request is missing RequestContext");
        assert!(
            matches!(req_context, &RequestContext::ApiGatewayV1(_)),
            "expected ApiGateway context, got {req_context:?}"
        );
    }

    #[test]
    fn deserializes_lambda_function_url_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/lambda/latest/dg/urls-invocation.html#urls-payloads
        let input = include_str!("../tests/data/lambda_function_url_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");
        let cookie_header = req
            .headers()
            .get_all(http::header::COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .reduce(|acc, nxt| [acc, nxt].join(";"));

        assert_eq!(req.method(), "GET");
        assert_eq!(
            req.uri(),
            "https://id.lambda-url.eu-west-2.on.aws/my/path?parameter1=value1&parameter1=value2&parameter2=value"
        );
        assert_eq!(cookie_header, Some("test=hi".to_string()));

        // Ensure this is an APIGWv2 request (Lambda Function URL requests confirm to API GW v2 Request format)
        let req_context = req.request_context_ref().expect("Request is missing RequestContext");
        assert!(
            matches!(req_context, &RequestContext::ApiGatewayV2(_)),
            "expected ApiGatewayV2 context, got {req_context:?}"
        );
    }

    #[test]
    fn deserializes_alb_request_events() {
        // from the docs
        // https://docs.aws.amazon.com/elasticloadbalancing/latest/application/lambda-functions.html#multi-value-headers
        let input = include_str!("../tests/data/alb_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(
            req.uri(),
            "https://lambda-846800462-us-east-2.elb.amazonaws.com/?myKey=val2"
        );

        // Ensure this is an ALB request
        let req_context = req.request_context_ref().expect("Request is missing RequestContext");
        assert!(
            matches!(req_context, &RequestContext::Alb(_)),
            "expected Alb context, got {req_context:?}"
        );
    }

    #[test]
    fn deserializes_alb_request_encoded_query_parameters_events() {
        // from the docs
        // https://docs.aws.amazon.com/elasticloadbalancing/latest/application/lambda-functions.html#multi-value-headers
        let input = include_str!("../tests/data/alb_request_encoded_query_parameters.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(
            req.uri(),
            "https://lambda-846800462-us-east-2.elb.amazonaws.com/?myKey=%3FshowAll%3Dtrue"
        );

        // Ensure this is an ALB request
        let req_context = req.request_context_ref().expect("Request is missing RequestContext");
        assert!(
            matches!(req_context, &RequestContext::Alb(_)),
            "expected Alb context, got {req_context:?}"
        );
    }

    #[test]
    fn deserializes_apigw_multi_value_request_events() {
        // from docs
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
        let input = include_str!("../tests/data/apigw_multi_value_proxy_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event is was not parsed as expected {result:?} given {input}"
        );
        let request = result.expect("failed to parse request");

        assert!(!request
            .query_string_parameters_ref()
            .expect("Request is missing query parameters")
            .is_empty());

        // test RequestExt#query_string_parameters_ref does the right thing
        let params = request.query_string_parameters();
        assert_eq!(Some(vec!["you", "me"]), params.all("multiValueName"));
        assert_eq!(Some(vec!["me"]), params.all("name"));

        let query = request.uri().query().unwrap();
        assert!(query.contains("name=me"));
        assert!(query.contains("multiValueName=you&multiValueName=me"));
        let (parts, _) = request.into_parts();
        assert!(parts.uri.to_string().contains("name=me"));
        assert!(parts.uri.to_string().contains("multiValueName=you&multiValueName=me"));
    }

    #[test]
    fn deserializes_alb_multi_value_request_events() {
        // from docs
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
        let input = include_str!("../tests/data/alb_multi_value_request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event is was not parsed as expected {result:?} given {input}"
        );
        let request = result.expect("failed to parse request");
        assert!(!request
            .query_string_parameters_ref()
            .expect("Request is missing query parameters")
            .is_empty());

        // test RequestExt#query_string_parameters_ref does the right thing
        let params = request.query_string_parameters();
        assert_eq!(Some(vec!["val1", "val2"]), params.all("myKey"));
        assert_eq!(Some(vec!["val3", "val4"]), params.all("myOtherKey"));

        let query = request.uri().query().unwrap();
        assert!(query.contains("myKey=val1&myKey=val2"));
        assert!(query.contains("myOtherKey=val3&myOtherKey=val4"));
    }

    #[test]
    fn deserializes_alb_multi_value_request_encoded_query_parameters_events() {
        // from docs
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
        let input = include_str!("../tests/data/alb_multi_value_request_encoded_query_parameters.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event is was not parsed as expected {result:?} given {input}"
        );
        let request = result.expect("failed to parse request");
        assert!(!request
            .query_string_parameters_ref()
            .expect("Request is missing query parameters")
            .is_empty());

        // test RequestExt#query_string_parameters_ref does the right thing
        assert_eq!(
            request
                .query_string_parameters_ref()
                .and_then(|params| params.all("myKey")),
            Some(vec!["?showAll=true", "?showAll=false"])
        );
    }

    #[test]
    fn deserialize_apigw_http_sam_local() {
        // manually generated from AWS SAM CLI
        // Steps to recreate:
        // * sam init
        // * Use, Zip Python 3.9, and Hello World example
        // * Change the template to use HttpApi instead of Api
        // * Change the function code to return the Lambda event serialized
        // * sam local start-api
        // * Invoke the API
        let input = include_str!("../tests/data/apigw_v2_sam_local.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(req.uri(), "http://127.0.0.1:3000/hello");
    }

    #[test]
    fn deserialize_apigw_no_host() {
        // generated from the 'apigateway-aws-proxy' test event template in the Lambda console
        let input = include_str!("../tests/data/apigw_no_host.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(req.uri(), "/test/hello?name=me");
    }

    #[test]
    fn deserialize_alb_no_host() {
        // generated from ALB health checks
        let input = include_str!("../tests/data/alb_no_host.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.method(), "GET");
        assert_eq!(req.uri(), "/v1/health/");
    }

    #[test]
    fn deserialize_apigw_path_with_space() {
        // generated from ALB health checks
        let input = include_str!("../tests/data/apigw_request_path_with_space.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");
        assert_eq!(req.uri(), "https://id.execute-api.us-east-1.amazonaws.com/my/path-with%20space?parameter1=value1&parameter1=value2&parameter2=value");
    }

    #[test]
    fn parse_paths_with_spaces() {
        let url = build_request_uri("/path with spaces/and multiple segments", &HeaderMap::new(), None, None);
        assert_eq!("/path%20with%20spaces/and%20multiple%20segments", url);
    }

    #[test]
    fn deserializes_apigw_http_request_with_stage_in_path() {
        let input = include_str!("../tests/data/apigw_v2_proxy_request_with_stage_in_path.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");
        assert_eq!("/Prod/my/path", req.uri().path());
        assert_eq!("/Prod/my/path", req.raw_http_path());
    }

    #[test]
    fn test_apigw_path_with_stage() {
        assert_eq!("/path", apigw_path_with_stage(&None, "/path"));
        assert_eq!("/path", apigw_path_with_stage(&Some("$default".into()), "/path"));
        assert_eq!("/Prod/path", apigw_path_with_stage(&Some("Prod".into()), "/Prod/path"));
        assert_eq!("/Prod/path", apigw_path_with_stage(&Some("Prod".into()), "/path"));
    }

    #[tokio::test]
    #[cfg(feature = "apigw_rest")]
    async fn test_axum_query_extractor_apigw_rest() {
        use axum_core::extract::FromRequestParts;
        use axum_extra::extract::Query;
        // from docs
        // https://docs.aws.amazon.com/apigateway/latest/developerguide/set-up-lambda-proxy-integrations.html#api-gateway-simple-proxy-for-lambda-input-format
        let input = include_str!("../tests/data/apigw_multi_value_proxy_request.json");
        let request = from_str(input).expect("failed to parse request");
        let (mut parts, _) = request.into_parts();

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Params {
            name: Vec<String>,
            multi_value_name: Vec<String>,
        }
        struct State;

        let query = Query::<Params>::from_request_parts(&mut parts, &State).await.unwrap();
        assert_eq!(vec!["me"], query.0.name);
        assert_eq!(vec!["you", "me"], query.0.multi_value_name);
    }

    #[tokio::test]
    #[cfg(feature = "apigw_http")]
    async fn test_axum_query_extractor_apigw_http() {
        use axum_core::extract::FromRequestParts;
        use axum_extra::extract::Query;
        let input = include_str!("../tests/data/apigw_v2_proxy_request.json");
        let request = from_str(input).expect("failed to parse request");
        let (mut parts, _) = request.into_parts();

        #[derive(Deserialize)]
        struct Params {
            parameter1: Vec<String>,
            parameter2: Vec<String>,
        }
        struct State;

        let query = Query::<Params>::from_request_parts(&mut parts, &State).await.unwrap();
        assert_eq!(vec!["value1", "value2"], query.0.parameter1);
        assert_eq!(vec!["value"], query.0.parameter2);
    }

    #[tokio::test]
    #[cfg(feature = "alb")]
    async fn test_axum_query_extractor_alb() {
        use axum_core::extract::FromRequestParts;
        use axum_extra::extract::Query;
        let input = include_str!("../tests/data/alb_multi_value_request.json");
        let request = from_str(input).expect("failed to parse request");
        let (mut parts, _) = request.into_parts();

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Params {
            my_key: Vec<String>,
            my_other_key: Vec<String>,
        }
        struct State;

        let query = Query::<Params>::from_request_parts(&mut parts, &State).await.unwrap();
        assert_eq!(vec!["val1", "val2"], query.0.my_key);
        assert_eq!(vec!["val3", "val4"], query.0.my_other_key);
    }

    #[test]
    #[cfg(feature = "apigw_rest")]
    fn deserializes_request_authorizer() {
        let input = include_str!("../../lambda-events/src/fixtures/example-apigw-request.json");
        let result = from_str(input);
        assert!(
            result.is_ok(),
            "event was not parsed as expected {result:?} given {input}"
        );
        let req = result.expect("failed to parse request");

        let req_context = req.request_context_ref().expect("Request is missing RequestContext");
        let authorizer = req_context.authorizer().expect("authorizer is missing");
        assert_eq!(Some("admin"), authorizer.fields.get("principalId").unwrap().as_str());
    }
}
