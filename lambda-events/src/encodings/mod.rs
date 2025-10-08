use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "chrono")]
mod time;
use crate::custom_serde::{deserialize_base64, serialize_base64};

#[cfg(feature = "chrono")]
#[cfg_attr(docsrs, doc(cfg(feature = "chrono")))]
pub use self::time::*;
#[cfg(feature = "http")]
mod http;
#[cfg(feature = "http")]
#[cfg_attr(docsrs, doc(cfg(feature = "http")))]
pub use self::http::*;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Binary data encoded in base64.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Base64Data(
    #[serde(deserialize_with = "deserialize_base64")]
    #[serde(serialize_with = "serialize_base64")]
    pub Vec<u8>,
);

impl Deref for Base64Data {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Base64Data {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
