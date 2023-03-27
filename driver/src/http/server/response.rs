use axum::http::header::SET_COOKIE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use cookie::Cookie;

pub fn response_with_code<T>(code: StatusCode, body: T) -> Response
where
    (StatusCode, T): IntoResponse,
{
    (code, body).into_response()
}

pub trait WithSetCookie {
    fn with_cookie(self, c: Cookie) -> Response;
}

impl WithSetCookie for Response {
    fn with_cookie(mut self, c: Cookie) -> Response {
        self.headers_mut().append(
            SET_COOKIE,
            HeaderValue::from_str(c.to_string().as_str()).unwrap(),
        );
        self
    }
}

pub mod constants {
    pub static SESSION_COOKIE_ID: &str = "sid";
    pub static TRACKING_COOKIE_ID: &str = "tid";
}
