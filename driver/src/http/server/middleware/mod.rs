use anyhow::bail;
use axum::headers::HeaderMap;
pub use axum::middleware::*;

use kernel::unexpected;

pub mod csrf;
pub mod request_id;
pub mod session;

pub(crate) fn get_header(headers: &HeaderMap, key: &str) -> kernel::Result<Option<String>> {
    let Some(value) = headers.get(key) else {
        return Ok(None)
    };
    let value = match value.to_str() {
        Ok(value) => value,
        Err(e) => bail!(unexpected!("header parse error: {}", e)),
    };
    Ok(Some(value.to_string()))
}

pub(crate) fn require_header(headers: &HeaderMap, key: &str) -> kernel::Result<String> {
    let Some(value) = get_header(headers, key)? else {
        bail!(unexpected!("require header not found: {}", key))
    };
    Ok(value)
}
