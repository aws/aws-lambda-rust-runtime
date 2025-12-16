//! Response types

use crate::request::RequestOrigin;
#[cfg(feature = "alb")]
use aws_lambda_events::alb::AlbTargetGroupResponse;
#[cfg(any(feature = "apigw_rest", feature = "apigw_websockets"))]
use aws_lambda_events::apigw::ApiGatewayProxyResponse;
#[cfg(feature = "apigw_http")]
use aws_lambda_events::apigw::ApiGatewayV2httpResponse;
use aws_lambda_events::encodings::Body;
use encoding_rs::Encoding;
use http::{
    header::{CONTENT_ENCODING, CONTENT_TYPE},
    HeaderMap, Response, StatusCode,
};
use http_body::Body as HttpBody;
use http_body_util::BodyExt;
use mime::{Mime, CHARSET};
use serde::Serialize;
use std::{
    borrow::Cow,
    fmt,
    future::{ready, Future},
    pin::Pin,
};

const X_LAMBDA_HTTP_CONTENT_ENCODING: &str = "x-lambda-http-content-encoding";

// See list of common MIME types:
// - https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
// - https://github.com/ietf-wg-httpapi/mediatypes/blob/main/draft-ietf-httpapi-yaml-mediatypes.md
const TEXT_ENCODING_PREFIXES: [&str; 5] = [
    "text",
    "application/json",
    "application/javascript",
    "application/xml",
    "application/yaml",
];

const TEXT_ENCODING_SUFFIXES: [&str; 3] = ["+xml", "+yaml", "+json"];

/// Representation of Lambda response
#[non_exhaustive]
#[doc(hidden)]
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum LambdaResponse {
    #[cfg(any(feature = "apigw_rest", feature = "apigw_websockets"))]
    ApiGatewayV1(ApiGatewayProxyResponse),
    #[cfg(feature = "apigw_http")]
    ApiGatewayV2(ApiGatewayV2httpResponse),
    #[cfg(feature = "alb")]
    Alb(AlbTargetGroupResponse),
    #[cfg(feature = "pass_through")]
    PassThrough(serde_json::Value),
}

/// Transformation from http type to internal type
impl LambdaResponse {
    pub(crate) fn from_response(request_origin: &RequestOrigin, value: Response<Body>) -> Self {
        let (parts, bod) = value.into_parts();
        let (is_base64_encoded, body) = match bod {
            Body::Empty => (false, None),
            b @ Body::Text(_) => (false, Some(b)),
            b @ Body::Binary(_) => (true, Some(b)),
            _ => (false, None),
        };

        let headers = parts.headers;
        let status_code = parts.status.as_u16();

        match request_origin {
            #[cfg(feature = "apigw_rest")]
            RequestOrigin::ApiGatewayV1 => LambdaResponse::ApiGatewayV1({
                let mut response = ApiGatewayProxyResponse::default();

                response.body = body;
                response.is_base64_encoded = is_base64_encoded;
                response.status_code = status_code as i64;
                // Explicitly empty, as API gateway v1 will merge "headers" and
                // "multi_value_headers" fields together resulting in duplicate response headers.
                response.headers = HeaderMap::new();
                response.multi_value_headers = headers;
                // Today, this implementation doesn't provide any additional fields
                #[cfg(feature = "catch-all-fields")]
                {
                    response.other = Default::default();
                }
                response
            }),
            #[cfg(feature = "apigw_http")]
            RequestOrigin::ApiGatewayV2 => {
                use http::header::SET_COOKIE;
                let mut headers = headers;
                // ApiGatewayV2 expects the set-cookies headers to be in the "cookies" attribute,
                // so remove them from the headers.
                let cookies = headers
                    .get_all(SET_COOKIE)
                    .iter()
                    .map(|v| v.to_str().ok().unwrap_or_default().to_string())
                    .collect();
                headers.remove(SET_COOKIE);

                LambdaResponse::ApiGatewayV2({
                    let mut response = ApiGatewayV2httpResponse::default();
                    response.body = body;
                    response.is_base64_encoded = is_base64_encoded;
                    response.status_code = status_code as i64;
                    response.cookies = cookies;
                    // API gateway v2 doesn't have "multi_value_headers" field. Duplicate headers
                    // are combined with commas and included in the headers field.
                    response.headers = headers;
                    response.multi_value_headers = HeaderMap::new();
                    // Today, this implementation doesn't provide any additional fields
                    #[cfg(feature = "catch-all-fields")]
                    {
                        response.other = Default::default();
                    }
                    response
                })
            }
            #[cfg(feature = "alb")]
            RequestOrigin::Alb => LambdaResponse::Alb({
                let mut response = AlbTargetGroupResponse::default();

                response.body = body;
                response.is_base64_encoded = is_base64_encoded;
                response.status_code = status_code as i64;
                // ALB responses are used for ALB integration, which can be configured to use
                // either "headers" or "multi_value_headers" field. We need to return both fields
                // to ensure both configuration work correctly.
                response.headers = headers.clone();
                response.multi_value_headers = headers;
                response.status_description = Some(format!(
                    "{} {}",
                    status_code,
                    parts.status.canonical_reason().unwrap_or_default()
                ));
                // Today, this implementation doesn't provide any additional fields
                #[cfg(feature = "catch-all-fields")]
                {
                    response.other = Default::default();
                }
                response
            }),
            #[cfg(feature = "apigw_websockets")]
            RequestOrigin::WebSocket => LambdaResponse::ApiGatewayV1({
                let mut response = ApiGatewayProxyResponse::default();
                response.body = body;
                response.is_base64_encoded = is_base64_encoded;
                response.status_code = status_code as i64;
                // Explicitly empty, as API gateway v1 will merge "headers" and
                // "multi_value_headers" fields together resulting in duplicate response headers.
                response.headers = HeaderMap::new();
                response.multi_value_headers = headers;
                // Today, this implementation doesn't provide any additional fields
                #[cfg(feature = "catch-all-fields")]
                {
                    response.other = Default::default();
                }
                response
            }),
            #[cfg(feature = "pass_through")]
            RequestOrigin::PassThrough => {
                match body {
                    // text body must be a valid json string
                    Some(Body::Text(body)) => {LambdaResponse::PassThrough(serde_json::from_str(&body).unwrap_or_default())},
                    // binary body and other cases return Value::Null
                    _ => LambdaResponse::PassThrough(serde_json::Value::Null),
                }
            }
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

/// Trait for generating responses
///
/// Types that implement this trait can be used as return types for handler functions.
pub trait IntoResponse {
    /// Transform into a `Response<Body>` Future
    fn into_response(self) -> ResponseFuture;
}

impl<B> IntoResponse for Response<B>
where
    B: ConvertBody + Send + 'static,
{
    fn into_response(self) -> ResponseFuture {
        let (parts, body) = self.into_parts();
        let headers = parts.headers.clone();

        let fut = async { Response::from_parts(parts, body.convert(headers).await) };

        Box::pin(fut)
    }
}

impl IntoResponse for String {
    fn into_response(self) -> ResponseFuture {
        Box::pin(ready(Response::new(Body::from(self))))
    }
}

impl IntoResponse for &str {
    fn into_response(self) -> ResponseFuture {
        Box::pin(ready(Response::new(Body::from(self))))
    }
}

impl IntoResponse for &[u8] {
    fn into_response(self) -> ResponseFuture {
        Box::pin(ready(Response::new(Body::from(self))))
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> ResponseFuture {
        Box::pin(ready(Response::new(Body::from(self))))
    }
}

impl IntoResponse for serde_json::Value {
    fn into_response(self) -> ResponseFuture {
        Box::pin(async move {
            Response::builder()
                .header(CONTENT_TYPE, "application/json")
                .body(
                    serde_json::to_string(&self)
                        .expect("unable to serialize serde_json::Value")
                        .into(),
                )
                .expect("unable to build http::Response")
        })
    }
}

impl IntoResponse for (StatusCode, String) {
    fn into_response(self) -> ResponseFuture {
        let (status, body) = self;
        Box::pin(ready(
            Response::builder()
                .status(status)
                .body(Body::from(body))
                .expect("unable to build http::Response"),
        ))
    }
}

impl IntoResponse for (StatusCode, &str) {
    fn into_response(self) -> ResponseFuture {
        let (status, body) = self;
        Box::pin(ready(
            Response::builder()
                .status(status)
                .body(Body::from(body))
                .expect("unable to build http::Response"),
        ))
    }
}

impl IntoResponse for (StatusCode, &[u8]) {
    fn into_response(self) -> ResponseFuture {
        let (status, body) = self;
        Box::pin(ready(
            Response::builder()
                .status(status)
                .body(Body::from(body))
                .expect("unable to build http::Response"),
        ))
    }
}

impl IntoResponse for (StatusCode, Vec<u8>) {
    fn into_response(self) -> ResponseFuture {
        let (status, body) = self;
        Box::pin(ready(
            Response::builder()
                .status(status)
                .body(Body::from(body))
                .expect("unable to build http::Response"),
        ))
    }
}

impl IntoResponse for (StatusCode, serde_json::Value) {
    fn into_response(self) -> ResponseFuture {
        let (status, body) = self;
        Box::pin(async move {
            Response::builder()
                .status(status)
                .header(CONTENT_TYPE, "application/json")
                .body(
                    serde_json::to_string(&body)
                        .expect("unable to serialize serde_json::Value")
                        .into(),
                )
                .expect("unable to build http::Response")
        })
    }
}

pub type ResponseFuture = Pin<Box<dyn Future<Output = Response<Body>> + Send>>;

pub trait ConvertBody {
    fn convert(self, parts: HeaderMap) -> BodyFuture;
}

impl<B> ConvertBody for B
where
    B: HttpBody + Unpin + Send + 'static,
    B::Data: Send,
    B::Error: fmt::Debug,
{
    fn convert(self, headers: HeaderMap) -> BodyFuture {
        if headers.get(CONTENT_ENCODING).is_some() {
            return convert_to_binary(self);
        }

        let content_type = if let Some(value) = headers.get(CONTENT_TYPE) {
            value.to_str().unwrap_or_default()
        } else {
            // Content-Type and Content-Encoding not set, passthrough as utf8 text
            return convert_to_text(self, "utf-8");
        };

        for prefix in TEXT_ENCODING_PREFIXES {
            if content_type.starts_with(prefix) {
                return convert_to_text(self, content_type);
            }
        }

        for suffix in TEXT_ENCODING_SUFFIXES {
            let mut parts = content_type.trim().split(';');
            let mime_type = parts.next().unwrap_or_default();
            if mime_type.ends_with(suffix) {
                return convert_to_text(self, content_type);
            }
        }

        if let Some(value) = headers.get(X_LAMBDA_HTTP_CONTENT_ENCODING) {
            if value == "text" {
                return convert_to_text(self, content_type);
            }
        }

        convert_to_binary(self)
    }
}

fn convert_to_binary<B>(body: B) -> BodyFuture
where
    B: HttpBody + Unpin + Send + 'static,
    B::Data: Send,
    B::Error: fmt::Debug,
{
    Box::pin(async move {
        Body::from(
            body.collect()
                .await
                .expect("unable to read bytes from body")
                .to_bytes()
                .to_vec(),
        )
    })
}

fn convert_to_text<B>(body: B, content_type: &str) -> BodyFuture
where
    B: HttpBody + Unpin + Send + 'static,
    B::Data: Send,
    B::Error: fmt::Debug,
{
    let mime_type = content_type.parse::<Mime>();

    let encoding = match mime_type.as_ref() {
        Ok(mime) => mime.get_param(CHARSET).unwrap_or(mime::UTF_8),
        Err(_) => mime::UTF_8,
    };

    let label = encoding.as_ref().as_bytes();
    let encoding = Encoding::for_label(label).unwrap_or(encoding_rs::UTF_8);

    // assumes utf-8
    Box::pin(async move {
        let bytes = body.collect().await.expect("unable to read bytes from body").to_bytes();
        let (content, _, _) = encoding.decode(&bytes);

        match content {
            Cow::Borrowed(content) => Body::from(content),
            Cow::Owned(content) => Body::from(content),
        }
    })
}

pub type BodyFuture = Pin<Box<dyn Future<Output = Body> + Send>>;

#[cfg(test)]
mod tests {
    use super::{Body, IntoResponse, LambdaResponse, RequestOrigin, X_LAMBDA_HTTP_CONTENT_ENCODING};
    use http::{
        header::{CONTENT_ENCODING, CONTENT_TYPE},
        Response, StatusCode,
    };
    use lambda_runtime_api_client::body::Body as HyperBody;
    use serde_json::{self, json};

    const SVG_LOGO: &str = include_str!("../tests/data/svg_logo.svg");

    #[tokio::test]
    async fn json_into_response() {
        let response = json!({ "hello": "lambda"}).into_response().await;
        match response.body() {
            Body::Text(json) => assert_eq!(json, r#"{"hello":"lambda"}"#),
            _ => panic!("invalid body"),
        }
        assert_eq!(
            response
                .headers()
                .get(CONTENT_TYPE)
                .map(|h| h.to_str().expect("invalid header")),
            Some("application/json")
        )
    }

    #[tokio::test]
    async fn text_into_response() {
        let response = "text".into_response().await;
        match response.body() {
            Body::Text(text) => assert_eq!(text, "text"),
            _ => panic!("invalid body"),
        }
    }

    #[tokio::test]
    async fn bytes_into_response() {
        let response = "text".as_bytes().into_response().await;
        match response.body() {
            Body::Binary(data) => assert_eq!(data, "text".as_bytes()),
            _ => panic!("invalid body"),
        }
    }

    #[tokio::test]
    async fn json_with_status_code_into_response() {
        let response = (StatusCode::CREATED, json!({ "hello": "lambda"})).into_response().await;
        match response.body() {
            Body::Text(json) => assert_eq!(json, r#"{"hello":"lambda"}"#),
            _ => panic!("invalid body"),
        }
        match response.status() {
            StatusCode::CREATED => (),
            _ => panic!("invalid status code"),
        }

        assert_eq!(
            response
                .headers()
                .get(CONTENT_TYPE)
                .map(|h| h.to_str().expect("invalid header")),
            Some("application/json")
        )
    }

    #[tokio::test]
    async fn text_with_status_code_into_response() {
        let response = (StatusCode::CREATED, "text").into_response().await;

        match response.status() {
            StatusCode::CREATED => (),
            _ => panic!("invalid status code"),
        }
        match response.body() {
            Body::Text(text) => assert_eq!(text, "text"),
            _ => panic!("invalid body"),
        }
    }

    #[tokio::test]
    async fn bytes_with_status_code_into_response() {
        let response = (StatusCode::CREATED, "text".as_bytes()).into_response().await;
        match response.status() {
            StatusCode::CREATED => (),
            _ => panic!("invalid status code"),
        }
        match response.body() {
            Body::Binary(data) => assert_eq!(data, "text".as_bytes()),
            _ => panic!("invalid body"),
        }
    }

    #[tokio::test]
    async fn content_encoding_header() {
        // Drive the implementation by using `hyper::Body` instead of
        // of `aws_lambda_events::encodings::Body`
        let response = Response::builder()
            .header(CONTENT_ENCODING, "gzip")
            .body(HyperBody::from("000000".as_bytes()))
            .expect("unable to build http::Response");
        let response = response.into_response().await;
        let response = LambdaResponse::from_response(&RequestOrigin::ApiGatewayV2, response);

        let json = serde_json::to_string(&response).expect("failed to serialize to json");
        assert_eq!(
            json,
            r#"{"statusCode":200,"headers":{"content-encoding":"gzip"},"multiValueHeaders":{},"body":"MDAwMDAw","isBase64Encoded":true,"cookies":[]}"#
        )
    }

    #[tokio::test]
    async fn content_type_header() {
        // Drive the implementation by using `hyper::Body` instead of
        // of `aws_lambda_events::encodings::Body`
        let response = Response::builder()
            .header(CONTENT_TYPE, "application/json")
            .body(HyperBody::from("000000".as_bytes()))
            .expect("unable to build http::Response");
        let response = response.into_response().await;
        let response = LambdaResponse::from_response(&RequestOrigin::ApiGatewayV2, response);

        let json = serde_json::to_string(&response).expect("failed to serialize to json");
        assert_eq!(
            json,
            r#"{"statusCode":200,"headers":{"content-type":"application/json"},"multiValueHeaders":{},"body":"000000","isBase64Encoded":false,"cookies":[]}"#
        )
    }

    #[tokio::test]
    async fn charset_content_type_header() {
        // Drive the implementation by using `hyper::Body` instead of
        // of `aws_lambda_events::encodings::Body`
        let response = Response::builder()
            .header(CONTENT_TYPE, "application/json; charset=utf-16")
            .body(HyperBody::from("000000".as_bytes()))
            .expect("unable to build http::Response");
        let response = response.into_response().await;
        let response = LambdaResponse::from_response(&RequestOrigin::ApiGatewayV2, response);

        let json = serde_json::to_string(&response).expect("failed to serialize to json");
        assert_eq!(
            json,
            r#"{"statusCode":200,"headers":{"content-type":"application/json; charset=utf-16"},"multiValueHeaders":{},"body":"〰〰〰","isBase64Encoded":false,"cookies":[]}"#
        )
    }

    #[tokio::test]
    async fn charset_content_type_header_suffix() {
        // Drive the implementation by using `hyper::Body` instead of
        // of `aws_lambda_events::encodings::Body`
        let response = Response::builder()
            .header(CONTENT_TYPE, "application/graphql-response+json; charset=utf-16")
            .body(HyperBody::from("000000".as_bytes()))
            .expect("unable to build http::Response");
        let response = response.into_response().await;
        let response = LambdaResponse::from_response(&RequestOrigin::ApiGatewayV2, response);

        let json = serde_json::to_string(&response).expect("failed to serialize to json");
        assert_eq!(
            json,
            r#"{"statusCode":200,"headers":{"content-type":"application/graphql-response+json; charset=utf-16"},"multiValueHeaders":{},"body":"〰〰〰","isBase64Encoded":false,"cookies":[]}"#
        )
    }

    #[tokio::test]
    async fn content_headers_unset() {
        // Drive the implementation by using `hyper::Body` instead of
        // of `aws_lambda_events::encodings::Body`
        let response = Response::builder()
            .body(HyperBody::from("000000".as_bytes()))
            .expect("unable to build http::Response");
        let response = response.into_response().await;
        let response = LambdaResponse::from_response(&RequestOrigin::ApiGatewayV2, response);

        let json = serde_json::to_string(&response).expect("failed to serialize to json");
        assert_eq!(
            json,
            r#"{"statusCode":200,"headers":{},"multiValueHeaders":{},"body":"000000","isBase64Encoded":false,"cookies":[]}"#
        )
    }

    #[test]
    fn serialize_multi_value_headers() {
        let res = LambdaResponse::from_response(
            &RequestOrigin::ApiGatewayV1,
            Response::builder()
                .header("multi", "a")
                .header("multi", "b")
                .body(Body::from(()))
                .expect("failed to create response"),
        );
        let json = serde_json::to_string(&res).expect("failed to serialize to json");
        assert_eq!(
            json,
            r#"{"statusCode":200,"headers":{},"multiValueHeaders":{"multi":["a","b"]},"isBase64Encoded":false}"#
        )
    }

    #[test]
    fn serialize_cookies() {
        let res = LambdaResponse::from_response(
            &RequestOrigin::ApiGatewayV2,
            Response::builder()
                .header("set-cookie", "cookie1=a")
                .header("set-cookie", "cookie2=b")
                .body(Body::from(()))
                .expect("failed to create response"),
        );
        let json = serde_json::to_string(&res).expect("failed to serialize to json");
        assert_eq!(
            "{\"statusCode\":200,\"headers\":{},\"multiValueHeaders\":{},\"isBase64Encoded\":false,\"cookies\":[\"cookie1=a\",\"cookie2=b\"]}",
            json
        )
    }

    #[tokio::test]
    async fn content_type_xml_as_text() {
        // Drive the implementation by using `hyper::Body` instead of
        // of `aws_lambda_events::encodings::Body`
        let response = Response::builder()
            .header(CONTENT_TYPE, "image/svg+xml")
            .body(HyperBody::from(SVG_LOGO.as_bytes()))
            .expect("unable to build http::Response");
        let response = response.into_response().await;

        match response.body() {
            Body::Text(body) => assert_eq!(SVG_LOGO, body),
            _ => panic!("invalid body"),
        }
        assert_eq!(
            response
                .headers()
                .get(CONTENT_TYPE)
                .map(|h| h.to_str().expect("invalid header")),
            Some("image/svg+xml")
        )
    }

    #[tokio::test]
    async fn content_type_custom_encoding_as_text() {
        // Drive the implementation by using `hyper::Body` instead of
        // of `aws_lambda_events::encodings::Body`
        let response = Response::builder()
            // this CONTENT-TYPE is not standard, and would yield a binary response
            .header(CONTENT_TYPE, "image/svg")
            .header(X_LAMBDA_HTTP_CONTENT_ENCODING, "text")
            .body(HyperBody::from(SVG_LOGO.as_bytes()))
            .expect("unable to build http::Response");
        let response = response.into_response().await;

        match response.body() {
            Body::Text(body) => assert_eq!(SVG_LOGO, body),
            _ => panic!("invalid body"),
        }
        assert_eq!(
            response
                .headers()
                .get(CONTENT_TYPE)
                .map(|h| h.to_str().expect("invalid header")),
            Some("image/svg")
        )
    }

    #[tokio::test]
    async fn content_type_yaml_as_text() {
        // Drive the implementation by using `hyper::Body` instead of
        // of `aws_lambda_events::encodings::Body`
        let yaml = r#"---
foo: bar
        "#;

        let formats = ["application/yaml", "custom/vdn+yaml"];

        for format in formats {
            let response = Response::builder()
                .header(CONTENT_TYPE, format)
                .body(HyperBody::from(yaml.as_bytes()))
                .expect("unable to build http::Response");
            let response = response.into_response().await;

            match response.body() {
                Body::Text(body) => assert_eq!(yaml, body),
                _ => panic!("invalid body"),
            }
            assert_eq!(
                response
                    .headers()
                    .get(CONTENT_TYPE)
                    .map(|h| h.to_str().expect("invalid header")),
                Some(format)
            )
        }
    }
}
