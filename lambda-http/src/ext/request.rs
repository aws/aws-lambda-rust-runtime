//! Extension methods for `Request` types

use std::{error::Error, fmt};

use serde::{
    de::{value::Error as SerdeError, DeserializeOwned},
    Deserialize,
};

use crate::Body;

/// Request payload deserialization errors
///
/// Returned by [`RequestPayloadExt::payload()`]
#[non_exhaustive]
#[derive(Debug)]
pub enum PayloadError {
    /// Returned when `application/json` bodies fail to deserialize a payload
    Json(serde_json::Error),
    /// Returned when `application/x-www-form-urlencoded` bodies fail to deserialize a payload
    WwwFormUrlEncoded(SerdeError),
}

/// Indicates a problem processing a JSON payload.
#[non_exhaustive]
#[derive(Debug)]
pub enum JsonPayloadError {
    /// Problem deserializing a JSON payload.
    Parsing(serde_json::Error),
}

/// Indicates a problem processing an x-www-form-urlencoded payload.
#[non_exhaustive]
#[derive(Debug)]
pub enum FormUrlEncodedPayloadError {
    /// Problem deserializing an x-www-form-urlencoded payload.
    Parsing(SerdeError),
}

impl From<JsonPayloadError> for PayloadError {
    fn from(err: JsonPayloadError) -> Self {
        match err {
            JsonPayloadError::Parsing(inner_err) => PayloadError::Json(inner_err),
        }
    }
}

impl From<FormUrlEncodedPayloadError> for PayloadError {
    fn from(err: FormUrlEncodedPayloadError) -> Self {
        match err {
            FormUrlEncodedPayloadError::Parsing(inner_err) => PayloadError::WwwFormUrlEncoded(inner_err),
        }
    }
}

impl fmt::Display for PayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PayloadError::Json(json) => writeln!(f, "failed to parse payload from application/json {json}"),
            PayloadError::WwwFormUrlEncoded(form) => writeln!(
                f,
                "failed to parse payload from application/x-www-form-urlencoded {form}"
            ),
        }
    }
}

impl Error for PayloadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PayloadError::Json(json) => Some(json),
            PayloadError::WwwFormUrlEncoded(form) => Some(form),
        }
    }
}

/// Extends `http::Request<Body>` with payload deserialization helpers.
pub trait RequestPayloadExt {
    /// Return the result of a payload parsed into a type that implements [`serde::Deserialize`]
    ///
    /// Currently only `application/x-www-form-urlencoded`
    /// and `application/json` flavors of content type
    /// are supported
    ///
    /// A [`PayloadError`] will be returned for undeserializable payloads.
    /// If no body is provided, the content-type header is missing,
    /// or the content-type header is unsupported, then `Ok(None)` will
    /// be returned. Note that a blank body (e.g. an empty string) is treated
    /// like a present payload by some deserializers and may result in an error.
    ///
    /// ### Examples
    ///
    /// A request's body can be deserialized if its correctly encoded as per
    /// the request's `Content-Type` header. The two supported content types are
    /// `application/x-www-form-urlencoded` and `application/json`.
    ///
    /// The following handler will work an http request body of `x=1&y=2`
    /// as well as `{"x":1, "y":2}` respectively.
    ///
    /// ```rust,no_run
    /// use lambda_http::{
    ///     service_fn, Body, Context, Error, IntoResponse, Request, RequestPayloadExt, Response,
    /// };
    /// use serde::Deserialize;
    ///
    /// #[derive(Debug, Default, Deserialize)]
    /// struct Args {
    ///   #[serde(default)]
    ///   x: usize,
    ///   #[serde(default)]
    ///   y: usize
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    ///   lambda_http::run(service_fn(add)).await?;
    ///   Ok(())
    /// }
    ///
    /// async fn add(
    ///   request: Request
    /// ) -> Result<Response<Body>, Error> {
    ///   let args: Args = request.payload()
    ///     .unwrap_or_else(|_parse_err| None)
    ///     .unwrap_or_default();
    ///   Ok(
    ///      Response::new(
    ///        format!(
    ///          "{} + {} = {}",
    ///          args.x,
    ///          args.y,
    ///          args.x + args.y
    ///        ).into()
    ///      )
    ///   )
    /// }
    fn payload<D>(&self) -> Result<Option<D>, PayloadError>
    where
        D: DeserializeOwned;

    /// Attempts to deserialize the request payload as JSON. When there is no payload,
    /// `Ok(None)` is returned.
    ///
    /// ### Errors
    ///
    /// If a present payload is not a valid JSON payload matching the annotated type,
    /// a [`JsonPayloadError`] is returned.
    ///
    /// ### Examples
    ///
    /// #### 1. Parsing a JSONString.
    /// ```ignore
    /// let req = http::Request::builder()
    ///     .body(Body::from("\"I am a JSON string\""))
    ///     .expect("failed to build request");
    /// match req.json::<String>() {
    ///     Ok(Some(json)) => assert_eq!(json, "I am a JSON string"),
    ///     Ok(None) => panic!("payload is missing."),
    ///     Err(err) => panic!("error processing json: {err:?}"),
    /// }
    /// ```
    ///
    /// #### 2. Parsing a JSONObject.
    /// ```ignore
    /// #[derive(Deserialize, Eq, PartialEq, Debug)]
    /// struct Person {
    ///     name: String,
    ///     age: u8,
    /// }
    ///
    /// let req = http::Request::builder()
    ///     .body(Body::from(r#"{"name": "Adam", "age": 23}"#))
    ///     .expect("failed to build request");
    ///
    /// match req.json::<Person>() {
    ///     Ok(Some(person)) => assert_eq!(
    ///         person,
    ///         Person {
    ///             name: "Adam".to_string(),
    ///             age: 23
    ///         }
    ///     ),
    ///     Ok(None) => panic!("payload is missing"),
    ///     Err(JsonPayloadError::Parsing(err)) => {
    ///         if err.is_data() {
    ///             panic!("payload does not match Person schema: {err:?}")
    ///         }
    ///         if err.is_syntax() {
    ///             panic!("payload is invalid json: {err:?}")
    ///         }
    ///         panic!("failed to parse json: {err:?}")
    ///     }
    /// }
    /// ```
    fn json<D>(&self) -> Result<Option<D>, JsonPayloadError>
    where
        D: DeserializeOwned;

    /// Attempts to deserialize the request payload as an application/x-www-form-urlencoded
    /// content type. When there is no payload, `Ok(None)` is returned.
    ///
    /// ### Errors
    ///
    /// If a present payload is not a valid application/x-www-form-urlencoded payload
    /// matching the annotated type, a [`FormUrlEncodedPayloadError`] is returned.
    ///
    /// ### Examples
    /// ```ignore
    /// let req = http::Request::builder()
    ///     .body(Body::from("name=Adam&age=23"))
    ///     .expect("failed to build request");
    /// match req.form_url_encoded::<Person>() {
    ///     Ok(Some(person)) => assert_eq!(
    ///         person,
    ///         Person {
    ///             name: "Adam".to_string(),
    ///             age: 23
    ///         }
    ///     ),
    ///     Ok(None) => panic!("payload is missing."),
    ///     Err(err) => panic!("error processing payload: {err:?}"),
    /// }
    /// ```
    fn form_url_encoded<D>(&self) -> Result<Option<D>, FormUrlEncodedPayloadError>
    where
        D: DeserializeOwned;
}

impl RequestPayloadExt for http::Request<Body> {
    fn payload<D>(&self) -> Result<Option<D>, PayloadError>
    where
        for<'de> D: Deserialize<'de>,
    {
        self.headers()
            .get(http::header::CONTENT_TYPE)
            .map(|ct| match ct.to_str() {
                Ok(content_type) => {
                    if content_type.starts_with("application/x-www-form-urlencoded") {
                        return self.form_url_encoded().map_err(PayloadError::from);
                    } else if content_type.starts_with("application/json") {
                        return self.json().map_err(PayloadError::from);
                    }
                    Ok(None)
                }
                _ => Ok(None),
            })
            .unwrap_or_else(|| Ok(None))
    }

    fn json<D>(&self) -> Result<Option<D>, JsonPayloadError>
    where
        D: DeserializeOwned,
    {
        if self.body().is_empty() {
            return Ok(None);
        }
        serde_json::from_slice::<D>(self.body().as_ref())
            .map(Some)
            .map_err(JsonPayloadError::Parsing)
    }

    fn form_url_encoded<D>(&self) -> Result<Option<D>, FormUrlEncodedPayloadError>
    where
        D: DeserializeOwned,
    {
        if self.body().is_empty() {
            return Ok(None);
        }
        serde_urlencoded::from_bytes::<D>(self.body().as_ref())
            .map(Some)
            .map_err(FormUrlEncodedPayloadError::Parsing)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::{FormUrlEncodedPayloadError, JsonPayloadError, RequestPayloadExt};

    use crate::Body;

    #[derive(Deserialize, Eq, PartialEq, Debug)]
    struct Payload {
        foo: String,
        baz: usize,
    }

    fn get_test_payload_as_json_body() -> Body {
        Body::from(r#"{"foo":"bar", "baz": 2}"#)
    }

    fn assert_eq_test_payload(payload: Option<Payload>) {
        assert_eq!(
            payload,
            Some(Payload {
                foo: "bar".into(),
                baz: 2
            })
        );
    }

    #[test]
    fn requests_have_form_post_parsable_payloads() {
        let request = http::Request::builder()
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from("foo=bar&baz=2"))
            .expect("failed to build request");
        let payload: Option<Payload> = request.payload().unwrap_or_default();
        assert_eq!(
            payload,
            Some(Payload {
                foo: "bar".into(),
                baz: 2
            })
        );
    }

    #[test]
    fn requests_have_json_parsable_payloads() {
        let request = http::Request::builder()
            .header("Content-Type", "application/json")
            .body(get_test_payload_as_json_body())
            .expect("failed to build request");
        let payload: Option<Payload> = request.payload().unwrap_or_default();
        assert_eq_test_payload(payload)
    }

    #[test]
    fn requests_match_form_post_content_type_with_charset() {
        let request = http::Request::builder()
            .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
            .body(Body::from("foo=bar&baz=2"))
            .expect("failed to build request");
        let payload: Option<Payload> = request.payload().unwrap_or_default();
        assert_eq!(
            payload,
            Some(Payload {
                foo: "bar".into(),
                baz: 2
            })
        );
    }

    #[test]
    fn requests_match_json_content_type_with_charset() {
        let request = http::Request::builder()
            .header("Content-Type", "application/json; charset=UTF-8")
            .body(Body::from(r#"{"foo":"bar", "baz": 2}"#))
            .expect("failed to build request");
        let payload: Option<Payload> = request.payload().unwrap_or_default();
        assert_eq!(
            payload,
            Some(Payload {
                foo: "bar".into(),
                baz: 2
            })
        );
    }

    #[test]
    fn requests_omitting_content_types_do_not_support_parsable_payloads() {
        let request = http::Request::builder()
            .body(Body::from(r#"{"foo":"bar", "baz": 2}"#))
            .expect("failed to build request");
        let payload: Option<Payload> = request.payload().unwrap_or_default();
        assert_eq!(payload, None);
    }

    #[test]
    fn requests_omitting_body_returns_none() {
        let request = http::Request::builder()
            .body(Body::Empty)
            .expect("failed to build request");
        let payload: Option<String> = request.payload().unwrap();
        assert_eq!(payload, None)
    }

    #[test]
    fn requests_with_json_content_type_hdr_omitting_body_returns_none() {
        let request = http::Request::builder()
            .header("Content-Type", "application/json; charset=UTF-8")
            .body(Body::Empty)
            .expect("failed to build request");
        let payload: Option<String> = request.payload().unwrap();
        assert_eq!(payload, None)
    }

    #[test]
    fn requests_with_formurlencoded_content_type_hdr_omitting_body_returns_none() {
        let request = http::Request::builder()
            .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
            .body(Body::Empty)
            .expect("failed to build request");
        let payload: Option<String> = request.payload().unwrap();
        assert_eq!(payload, None)
    }

    #[derive(Deserialize, Eq, PartialEq, Debug)]
    struct Person {
        name: String,
        age: u8,
    }

    #[test]
    fn json_fn_parses_json_strings() {
        let req = http::Request::builder()
            .body(Body::from("\"I am a JSON string\""))
            .expect("failed to build request");
        match req.json::<String>() {
            Ok(Some(json)) => assert_eq!(json, "I am a JSON string"),
            Ok(None) => panic!("payload is missing."),
            Err(err) => panic!("error processing json: {err:?}"),
        }
    }

    #[test]
    fn json_fn_parses_objects() {
        let req = http::Request::builder()
            .body(Body::from(r#"{"name": "Adam", "age": 23}"#))
            .expect("failed to build request");

        match req.json::<Person>() {
            Ok(Some(person)) => assert_eq!(
                person,
                Person {
                    name: "Adam".to_string(),
                    age: 23
                }
            ),
            Ok(None) => panic!("request data missing"),
            Err(JsonPayloadError::Parsing(err)) => {
                if err.is_data() {
                    panic!("payload does not match Person: {err:?}")
                }
                if err.is_syntax() {
                    panic!("invalid json: {err:?}")
                }
                panic!("failed to parse json: {err:?}")
            }
        }
    }

    #[test]
    fn json_fn_parses_list_of_objects() {
        let req = http::Request::builder()
            .body(Body::from(
                r#"[{"name": "Adam", "age": 23}, {"name": "Sarah", "age": 47}]"#,
            ))
            .expect("failed to build request");
        let expected_result = vec![
            Person {
                name: "Adam".to_string(),
                age: 23,
            },
            Person {
                name: "Sarah".to_string(),
                age: 47,
            },
        ];
        let result: Vec<Person> = req.json().expect("invalid payload").expect("missing payload");
        assert_eq!(result, expected_result);
    }

    #[test]
    fn json_fn_parses_nested_objects() {
        #[derive(Deserialize, Eq, PartialEq, Debug)]
        struct Pet {
            name: String,
            owner: Person,
        }

        let req = http::Request::builder()
            .body(Body::from(
                r#"{"name": "Gumball", "owner": {"name": "Adam", "age": 23}}"#,
            ))
            .expect("failed to build request");

        let expected_result = Pet {
            name: "Gumball".to_string(),
            owner: Person {
                name: "Adam".to_string(),
                age: 23,
            },
        };
        let result: Pet = req.json().expect("invalid payload").expect("missing payload");
        assert_eq!(result, expected_result);
    }

    #[test]
    fn json_fn_accepts_request_with_content_type_header() {
        let request = http::Request::builder()
            .header("Content-Type", "application/json")
            .body(get_test_payload_as_json_body())
            .expect("failed to build request");
        let payload: Option<Payload> = request.json().unwrap();
        assert_eq_test_payload(payload)
    }

    #[test]
    fn json_fn_accepts_request_without_content_type_header() {
        let request = http::Request::builder()
            .body(get_test_payload_as_json_body())
            .expect("failed to build request");
        let payload: Option<Payload> = request.json().expect("failed to parse json");
        assert_eq_test_payload(payload)
    }

    #[test]
    fn json_fn_given_nonjson_payload_returns_syntax_error() {
        let request = http::Request::builder()
            .body(Body::Text(String::from("Not a JSON")))
            .expect("failed to build request");
        let payload = request.json::<String>();
        assert!(payload.is_err());

        if let Err(JsonPayloadError::Parsing(err)) = payload {
            assert!(err.is_syntax())
        } else {
            panic!(
                "{}",
                format!("payload should have caused a parsing error. instead, it was {payload:?}")
            );
        }
    }

    #[test]
    fn json_fn_given_unexpected_payload_shape_returns_data_error() {
        let request = http::Request::builder()
            .body(Body::from(r#"{"foo":"bar", "baz": "!SHOULD BE A NUMBER!"}"#))
            .expect("failed to build request");
        let result = request.json::<Payload>();

        if let Err(JsonPayloadError::Parsing(err)) = result {
            assert!(err.is_data())
        } else {
            panic!(
                "{}",
                format!("payload should have caused a parsing error. instead, it was {result:?}")
            );
        }
    }

    #[test]
    fn json_fn_given_empty_payload_returns_none() {
        let empty_request = http::Request::default();
        let payload: Option<Payload> = empty_request.json().expect("failed to parse json");
        assert_eq!(payload, None)
    }

    #[test]
    fn form_url_encoded_fn_parses_forms() {
        let req = http::Request::builder()
            .body(Body::from("name=Adam&age=23"))
            .expect("failed to build request");
        match req.form_url_encoded::<Person>() {
            Ok(Some(person)) => assert_eq!(
                person,
                Person {
                    name: "Adam".to_string(),
                    age: 23
                }
            ),
            Ok(None) => panic!("payload is missing."),
            Err(err) => panic!("error processing payload: {err:?}"),
        }
    }

    #[test]
    fn form_url_encoded_fn_accepts_request_with_content_type_header() {
        let request = http::Request::builder()
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(Body::from("foo=bar&baz=2"))
            .expect("failed to build request");
        let payload: Option<Payload> = request.form_url_encoded().unwrap();
        assert_eq_test_payload(payload);
    }

    #[test]
    fn form_url_encoded_fn_accepts_request_without_content_type_header() {
        let request = http::Request::builder()
            .body(Body::from("foo=bar&baz=2"))
            .expect("failed to build request");
        let payload: Option<Payload> = request.form_url_encoded().expect("failed to parse form");
        assert_eq_test_payload(payload);
    }

    #[test]
    fn form_url_encoded_fn_given_non_form_urlencoded_payload_errors() {
        let request = http::Request::builder()
            .body(Body::Text(String::from("Not a url-encoded form")))
            .expect("failed to build request");
        let payload = request.form_url_encoded::<String>();
        assert!(payload.is_err());
        assert!(matches!(payload, Err(FormUrlEncodedPayloadError::Parsing(_))));
    }

    #[test]
    fn form_url_encoded_fn_given_unexpected_payload_shape_errors() {
        let request = http::Request::builder()
            .body(Body::from("foo=bar&baz=SHOULD_BE_A_NUMBER"))
            .expect("failed to build request");
        let result = request.form_url_encoded::<Payload>();
        assert!(result.is_err());
        assert!(matches!(result, Err(FormUrlEncodedPayloadError::Parsing(_))));
    }

    #[test]
    fn form_url_encoded_fn_given_empty_payload_returns_none() {
        let empty_request = http::Request::default();
        let payload: Option<Payload> = empty_request.form_url_encoded().expect("failed to parse form");
        assert_eq!(payload, None);
    }
}
