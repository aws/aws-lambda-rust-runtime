use base64::display::Base64Display;
use bytes::Bytes;
use http_body::{Body as HttpBody, SizeHint};
use serde::{
    de::{Deserialize, Deserializer, Error as DeError, Visitor},
    ser::{Error as SerError, Serialize, Serializer},
};
use std::{borrow::Cow, mem::take, ops::Deref, pin::Pin, task::Poll};

/// Representation of http request and response bodies as supported
/// by API Gateway and ALBs.
///
/// These come in three flavors
/// * `Empty` ( no body )
/// * `Text` ( text data )
/// * `Binary` ( binary data )
///
/// Body types can be `Deref` and `AsRef`'d into `[u8]` types much like the [hyper crate](https://crates.io/crates/hyper)
///
/// # Examples
///
/// Body types are inferred with `From` implementations.
///
/// ## Text
///
/// Types like `String`, `str` whose type reflects
/// text produce `Body::Text` variants
///
/// ```
/// assert!(match aws_lambda_events::encodings::Body::from("text") {
///   aws_lambda_events::encodings::Body::Text(_) => true,
///   _ => false
/// })
/// ```
///
/// ## Binary
///
/// Types like `Vec<u8>` and `&[u8]` whose types reflect raw bytes produce `Body::Binary` variants
///
/// ```
/// assert!(match aws_lambda_events::encodings::Body::from("text".as_bytes()) {
///   aws_lambda_events::encodings::Body::Binary(_) => true,
///   _ => false
/// })
/// ```
///
/// `Binary` responses bodies will automatically get based64 encoded to meet API Gateway's response expectations.
///
/// ## Empty
///
/// The unit type ( `()` ) whose type represents an empty value produces `Body::Empty` variants
///
/// ```
/// assert!(match aws_lambda_events::encodings::Body::from(()) {
///   aws_lambda_events::encodings::Body::Empty => true,
///   _ => false
/// })
/// ```
///
///
/// For more information about API Gateway's body types,
/// refer to [this documentation](https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-payload-encodings.html).
#[non_exhaustive]
#[derive(Debug, Default, Eq, PartialEq)]
pub enum Body {
    /// An empty body
    #[default]
    Empty,
    /// A body containing string data
    Text(String),
    /// A body containing binary data
    Binary(Vec<u8>),
}

impl Body {
    /// Decodes body, if needed.
    ///
    /// # Panics
    ///
    /// Panics when aws communicates to handler that request is base64 encoded but
    /// it can not be base64 decoded
    pub fn from_maybe_encoded(is_base64_encoded: bool, body: &str) -> Body {
        use base64::Engine;

        if is_base64_encoded {
            Body::from(
                ::base64::engine::general_purpose::STANDARD
                    .decode(body)
                    .expect("failed to decode aws base64 encoded body"),
            )
        } else {
            Body::from(body)
        }
    }
}

impl From<()> for Body {
    fn from(_: ()) -> Self {
        Body::Empty
    }
}

impl<'a> From<&'a str> for Body {
    fn from(s: &'a str) -> Self {
        Body::Text(s.into())
    }
}

impl From<String> for Body {
    fn from(b: String) -> Self {
        Body::Text(b)
    }
}

impl From<Cow<'static, str>> for Body {
    #[inline]
    fn from(cow: Cow<'static, str>) -> Body {
        match cow {
            Cow::Borrowed(b) => Body::from(b.to_owned()),
            Cow::Owned(o) => Body::from(o),
        }
    }
}

impl From<Cow<'static, [u8]>> for Body {
    #[inline]
    fn from(cow: Cow<'static, [u8]>) -> Body {
        match cow {
            Cow::Borrowed(b) => Body::from(b),
            Cow::Owned(o) => Body::from(o),
        }
    }
}

impl From<Vec<u8>> for Body {
    fn from(b: Vec<u8>) -> Self {
        Body::Binary(b)
    }
}

impl<'a> From<&'a [u8]> for Body {
    fn from(b: &'a [u8]) -> Self {
        Body::Binary(b.to_vec())
    }
}

impl Deref for Body {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl AsRef<[u8]> for Body {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        match self {
            Body::Empty => &[],
            Body::Text(ref bytes) => bytes.as_ref(),
            Body::Binary(ref bytes) => bytes.as_ref(),
        }
    }
}

impl Clone for Body {
    fn clone(&self) -> Self {
        match self {
            Body::Empty => Body::Empty,
            Body::Text(ref bytes) => Body::Text(bytes.clone()),
            Body::Binary(ref bytes) => Body::Binary(bytes.clone()),
        }
    }
}

impl Serialize for Body {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Body::Text(data) => {
                serializer.serialize_str(::std::str::from_utf8(data.as_ref()).map_err(S::Error::custom)?)
            }
            Body::Binary(data) => {
                serializer.collect_str(&Base64Display::new(data, &base64::engine::general_purpose::STANDARD))
            }
            Body::Empty => serializer.serialize_unit(),
        }
    }
}

impl<'de> Deserialize<'de> for Body {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BodyVisitor;

        impl Visitor<'_> for BodyVisitor {
            type Value = Body;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Body, E>
            where
                E: DeError,
            {
                Ok(Body::from(value))
            }
        }

        deserializer.deserialize_str(BodyVisitor)
    }
}

impl HttpBody for Body {
    type Data = Bytes;
    type Error = super::Error;

    fn is_end_stream(&self) -> bool {
        matches!(self, Body::Empty)
    }

    fn size_hint(&self) -> SizeHint {
        match self {
            Body::Empty => SizeHint::default(),
            Body::Text(ref s) => SizeHint::with_exact(s.len() as u64),
            Body::Binary(ref b) => SizeHint::with_exact(b.len() as u64),
        }
    }

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let body = take(self.get_mut());
        Poll::Ready(match body {
            Body::Empty => None,
            Body::Text(s) => Some(Ok(http_body::Frame::data(s.into()))),
            Body::Binary(b) => Some(Ok(http_body::Frame::data(b.into()))),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn body_has_default() {
        assert_eq!(Body::default(), Body::Empty);
    }

    #[test]
    fn from_unit() {
        assert_eq!(Body::from(()), Body::Empty);
    }

    #[test]
    fn from_str() {
        match Body::from(String::from("foo").as_str()) {
            Body::Text(_) => (),
            not => panic!("expected Body::Text(...) got {not:?}"),
        }
    }

    #[test]
    fn from_string() {
        match Body::from(String::from("foo")) {
            Body::Text(_) => (),
            not => panic!("expected Body::Text(...) got {not:?}"),
        }
    }

    #[test]
    fn from_cow_str() {
        match Body::from(Cow::from("foo")) {
            Body::Text(_) => (),
            not => panic!("expected Body::Text(...) got {not:?}"),
        }
    }

    #[test]
    fn from_cow_bytes() {
        match Body::from(Cow::from("foo".as_bytes())) {
            Body::Binary(_) => (),
            not => panic!("expected Body::Binary(...) got {not:?}"),
        }
    }

    #[test]
    fn from_bytes() {
        match Body::from("foo".as_bytes()) {
            Body::Binary(_) => (),
            not => panic!("expected Body::Binary(...) got {not:?}"),
        }
    }

    #[test]
    fn serialize_text() {
        let mut map = HashMap::new();
        map.insert("foo", Body::from("bar"));
        assert_eq!(serde_json::to_string(&map).unwrap(), r#"{"foo":"bar"}"#);
    }

    #[test]
    fn serialize_binary() {
        let mut map = HashMap::new();
        map.insert("foo", Body::from("bar".as_bytes()));
        assert_eq!(serde_json::to_string(&map).unwrap(), r#"{"foo":"YmFy"}"#);
    }

    #[test]
    fn serialize_empty() {
        let mut map = HashMap::new();
        map.insert("foo", Body::Empty);
        assert_eq!(serde_json::to_string(&map).unwrap(), r#"{"foo":null}"#);
    }

    #[test]
    fn serialize_from_maybe_encoded() {
        match Body::from_maybe_encoded(false, "foo") {
            Body::Text(_) => (),
            not => panic!("expected Body::Text(...) got {not:?}"),
        }

        match Body::from_maybe_encoded(true, "Zm9v") {
            Body::Binary(b) => assert_eq!(&[102, 111, 111], b.as_slice()),
            not => panic!("expected Body::Text(...) got {not:?}"),
        }
    }
}
