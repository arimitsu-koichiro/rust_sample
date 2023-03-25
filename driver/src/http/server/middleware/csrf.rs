use crate::http::server::middleware::{get_header, require_header};
use crate::http::server::response::response_with_code;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use helper::env::get_var;
use kernel::Result;
use log;

pub(crate) async fn csrf_protection<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, CSRFError> {
    // TODO refactor to interactor
    if CSRF_CHECK_WHITELIST.contains(&request.uri().path()) {
        return Ok(next.run(request).await);
    }
    let headers = request.headers();
    let env_hosts = get_var::<String>("CSRF_ALLOW_SITE_HOSTS")?;
    let host = require_header(headers, "host")?;
    if !env_hosts.contains(&host) {
        return Ok(response_with_code(
            StatusCode::FORBIDDEN,
            format!("invalid host: {host}"),
        ));
    }
    let is_websocket = get_header(headers, "sec-websocket-protocol")?.is_some();
    let is_event_stream = get_header(headers, "accept")? == Some("text/event-stream".to_string());
    let x_from = get_header(headers, "x-from")?;
    let env_urls = get_var::<String>("CSRF_ALLOW_X_FROM")?;
    match x_from {
        Some(_) => match get_header(headers, "origin")? {
            Some(origin) if !env_urls.contains(&origin) => {
                return Ok(response_with_code(
                    StatusCode::FORBIDDEN,
                    format!("invalid origin: {origin}"),
                ));
            }
            _ => (),
        },
        _ if !is_websocket && !is_event_stream => {
            return Ok(response_with_code(
                StatusCode::FORBIDDEN,
                format!("invalid x-from: {x_from:?}"),
            ))
        }
        _ => (),
    }
    Ok(next.run(request).await)
}

#[derive(Debug)]
pub struct CSRFError(anyhow::Error);

impl IntoResponse for CSRFError {
    fn into_response(self) -> Response {
        log::error!("{:?}", self.0);
        response_with_code(StatusCode::INTERNAL_SERVER_ERROR, "InternalServerError")
    }
}

impl From<anyhow::Error> for CSRFError {
    fn from(err: anyhow::Error) -> Self {
        CSRFError(err)
    }
}
const CSRF_CHECK_WHITELIST: &[&str] = &["/api/v1/status"];
