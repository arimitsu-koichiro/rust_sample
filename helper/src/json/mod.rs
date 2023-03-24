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
    serde_json::from_slice::<'a, T>(value).with_context(|| "json::helper::from_bytes error")
}

pub fn from_vec<'a, T: Deserialize<'a>>(value: &'a [u8]) -> Result<T> {
    serde_json::from_slice::<'a, T>(value).with_context(|| "json::helper::from_vec error")
}
