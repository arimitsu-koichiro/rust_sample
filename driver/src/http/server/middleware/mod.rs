use anyhow::bail;
use axum::headers::{HeaderMap, HeaderName};
pub use axum::middleware::*;

use kernel::unexpected;

pub mod csrf;
pub mod request_id;
pub mod session;
pub mod tracking;

pub(crate) fn get_header(
    headers: &HeaderMap,
    header_name: &HeaderName,
) -> kernel::Result<Option<String>> {
    let Some(value) = headers.get(header_name) else {
        return Ok(None)
    };
    let value = match value.to_str() {
        Ok(value) => value,
        Err(e) => bail!(unexpected!("header parse error: {}", e)),
    };
    Ok(Some(value.to_string()))
}

pub(crate) fn require_header(
    headers: &HeaderMap,
    header_name: &HeaderName,
) -> kernel::Result<String> {
    let Some(value) = get_header(headers, header_name)? else {
        bail!(unexpected!("require header not found: {}", header_name))
    };
    Ok(value)
}
