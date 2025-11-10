use http::{header::HeaderName, HeaderMap, HeaderValue};
use serde::{
    de::{self, Deserializer, Error as DeError, MapAccess, Unexpected, Visitor},
    ser::{SerializeMap, Serializer},
};
use std::{borrow::Cow, fmt};

/// Deserialize (potentially) comma separated headers into a HeaderMap
pub(crate) fn deserialize_comma_separated_headers<'de, D>(de: D) -> Result<HeaderMap, D::Error>
where
    D: Deserializer<'de>,
{
    let is_human_readable = de.is_human_readable();
    de.deserialize_option(HeaderMapVisitor { is_human_readable })
}

/// Serialize a HeaderMap with multiple values per header combined as comma-separated strings
pub(crate) fn serialize_comma_separated_headers<S>(headers: &HeaderMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(headers.keys_len()))?;

    // Group headers by name and combine values
    for key in headers.keys() {
        let values: Vec<&str> = headers
            .get_all(key)
            .iter()
            .filter_map(|v| v.to_str().ok()) // Skip invalid UTF-8 values
            .collect();

        if !values.is_empty() {
            let combined_value = values.join(", ");
            map.serialize_entry(key.as_str(), &combined_value)?;
        }
    }

    map.end()
}

// extension/duplicate of existing code from custom_serde/headers.rs
// could possibly be refactored back into common code

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum OneOrMore<'a> {
    One(Cow<'a, str>),
    Strings(Vec<Cow<'a, str>>),
    Bytes(Vec<Cow<'a, [u8]>>),
    CommaSeparated(Cow<'a, str>),
}

struct HeaderMapVisitor {
    is_human_readable: bool,
}

impl<'de> Visitor<'de> for HeaderMapVisitor {
    type Value = HeaderMap;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("lots of things can go wrong with HeaderMap")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(HeaderMap::default())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(HeaderMap::default())
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map = HeaderMap::with_capacity(access.size_hint().unwrap_or(0));

        if !self.is_human_readable {
            while let Some((key, arr)) = access.next_entry::<Cow<'_, str>, Vec<Cow<'_, [u8]>>>()? {
                let key = HeaderName::from_bytes(key.as_bytes())
                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(&key), &self))?;
                for val in arr {
                    let val = HeaderValue::from_bytes(&val)
                        .map_err(|_| de::Error::invalid_value(Unexpected::Bytes(&val), &self))?;
                    map.append(&key, val);
                }
            }
        } else {
            while let Some((key, val)) = access.next_entry::<Cow<'_, str>, OneOrMore<'_>>()? {
                let key = HeaderName::from_bytes(key.as_bytes())
                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(&key), &self))?;
                match val {
                    OneOrMore::One(val) => {
                        // Check if the single value contains commas and split if needed
                        if val.contains(',') {
                            split_and_append_header(&mut map, &key, &val, &self)?;
                        } else {
                            let header_val = val
                                .parse()
                                .map_err(|_| de::Error::invalid_value(Unexpected::Str(&val), &self))?;
                            map.insert(key, header_val);
                        }
                    }
                    OneOrMore::Strings(arr) => {
                        for val in arr {
                            // Each string in the array might also be comma-separated
                            if val.contains(',') {
                                split_and_append_header(&mut map, &key, &val, &self)?;
                            } else {
                                let header_val = val
                                    .parse()
                                    .map_err(|_| de::Error::invalid_value(Unexpected::Str(&val), &self))?;
                                map.append(&key, header_val);
                            }
                        }
                    }
                    OneOrMore::Bytes(arr) => {
                        for val in arr {
                            let header_val = HeaderValue::from_bytes(&val)
                                .map_err(|_| de::Error::invalid_value(Unexpected::Bytes(&val), &self))?;
                            map.append(&key, header_val);
                        }
                    }
                    OneOrMore::CommaSeparated(val) => {
                        // Explicitly handle comma-separated values
                        split_and_append_header(&mut map, &key, &val, &self)?;
                    }
                };
            }
        }
        Ok(map)
    }
}

fn split_and_append_header<E>(
    map: &mut HeaderMap,
    key: &HeaderName,
    value: &str,
    visitor: &HeaderMapVisitor,
) -> Result<(), E>
where
    E: DeError,
{
    for split_val in value.split(',') {
        let trimmed_val = split_val.trim();
        if !trimmed_val.is_empty() {
            // Skip empty values from trailing commas
            let header_val = trimmed_val
                .parse()
                .map_err(|_| de::Error::invalid_value(Unexpected::Str(trimmed_val), visitor))?;
            map.append(key, header_val);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::{HeaderMap, HeaderValue};
    use serde_json;
    use serde_with::serde_derive::{Deserialize, Serialize};

    #[test]
    fn test_function_deserializer() {
        #[derive(Deserialize)]
        struct RequestWithHeaders {
            #[serde(deserialize_with = "deserialize_comma_separated_headers")]
            headers: HeaderMap,
        }

        let r: RequestWithHeaders =
            serde_json::from_str("{ \"headers\": {\"x-foo\": \"z\", \"x-multi\": \"abcd, DEF, w\" }}").unwrap();

        assert_eq!("z", r.headers.get_all("x-foo").iter().next().unwrap());
        assert_eq!("abcd", r.headers.get_all("x-multi").iter().next().unwrap());
        assert_eq!("DEF", r.headers.get_all("x-multi").iter().nth(1).unwrap());
        assert_eq!("w", r.headers.get_all("x-multi").iter().nth(2).unwrap());
    }

    fn create_test_headermap() -> HeaderMap {
        let mut headers = HeaderMap::new();

        // Single value header
        headers.insert("content-type", HeaderValue::from_static("application/json"));

        // Multiple value header
        headers.append("accept", HeaderValue::from_static("text/html"));
        headers.append("accept", HeaderValue::from_static("application/json"));
        headers.append("accept", HeaderValue::from_static("*/*"));

        // Another multiple value header
        headers.append("cache-control", HeaderValue::from_static("no-cache"));
        headers.append("cache-control", HeaderValue::from_static("must-revalidate"));

        headers
    }

    #[test]
    fn test_function_serializer() {
        #[derive(Serialize)]
        struct RequestWithHeaders {
            #[serde(serialize_with = "serialize_comma_separated_headers")]
            headers: HeaderMap,
            body: String,
        }

        let request = RequestWithHeaders {
            headers: create_test_headermap(),
            body: "test body".to_string(),
        };

        let json = serde_json::to_string_pretty(&request).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed["headers"]["accept"].as_str().unwrap().contains(", "));
    }
}
