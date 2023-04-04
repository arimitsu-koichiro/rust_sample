use anyhow::{Context as _, Result};
use serde;
use serde::{Deserialize, Serialize};
use serde_json;

pub fn to_string<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string::<T>(value).with_context(|| "json::helper::to_string error")
}

pub fn to_string_pretty<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string_pretty::<T>(value).with_context(|| "json::helper::to_string_pretty error")
}

pub fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec::<T>(value).with_context(|| "json::helper::to_vec error")
}

pub fn to_vec_pretty<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    serde_json::to_vec_pretty::<T>(value).with_context(|| "json::helper::to_vec_pretty error")
}

pub fn from_string<'a, T: Deserialize<'a>>(value: &'a str) -> Result<T> {
    serde_json::from_str::<'a, T>(value).with_context(|| "json::helper::from_string error")
}

pub fn from_bytes<'a, T: Deserialize<'a>>(value: &'a [u8]) -> Result<T> {
    serde_json::from_slice::<'a, T>(value).with_context(|| "json::helper::from_vec error")
}

pub trait ToJson<T: Serialize> {
    fn to_json_string(&self) -> Result<String>;
    fn to_json_string_pretty(&self) -> Result<String>;
    fn to_json_vec(&self) -> Result<Vec<u8>>;
    fn to_json_vec_pretty(&self) -> Result<Vec<u8>>;
}

impl<T: Serialize> ToJson<T> for T {
    fn to_json_string(&self) -> Result<String> {
        to_string(self)
    }

    fn to_json_string_pretty(&self) -> Result<String> {
        to_string_pretty(self)
    }

    fn to_json_vec(&self) -> Result<Vec<u8>> {
        to_vec(self)
    }

    fn to_json_vec_pretty(&self) -> Result<Vec<u8>> {
        to_vec_pretty(self)
    }
}

pub trait FromJson {
    fn deserialize<'a, 'b: 'a, T: Deserialize<'a>>(&'b self) -> Result<T>;
}

impl FromJson for &str {
    fn deserialize<'a, 'b: 'a, T: Deserialize<'a>>(&'b self) -> Result<T> {
        from_string::<T>(self)
    }
}

impl FromJson for &[u8] {
    fn deserialize<'a, 'b: 'a, T: Deserialize<'a>>(&'b self) -> Result<T> {
        from_bytes::<T>(self)
    }
}

impl FromJson for Vec<u8> {
    fn deserialize<'a, 'b: 'a, T: Deserialize<'a>>(&'b self) -> Result<T> {
        from_bytes::<T>(self)
    }
}
