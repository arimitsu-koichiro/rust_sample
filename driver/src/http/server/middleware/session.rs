use crate::http::server::response::constants;

use application::usecase::session::{GetSessionInput, GetSessionOutput};
use application::usecase::{UseCase, UseUseCase};
use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header::COOKIE;
use axum::http::request::Parts;
use axum::http::StatusCode;
use cookie::Cookie;
use kernel::entity::Session;

pub struct ExtractSession(pub Option<Session>);
pub struct RequireSession(pub Session);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractSession
where
    S: Send + Sync,
    S: UseUseCase<GetSessionInput, GetSessionOutput>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        for cookie in parts.headers.get_all(COOKIE) {
            let Ok(cookie) = cookie.to_str() else {
                continue
            };
            let Ok(cookie) = Cookie::parse(cookie) else {
                continue
            };
            if cookie.name() != constants::SESSION_COOKIE_ID {
                continue;
            }
            return match state
                .usecase()
                .handle(GetSessionInput::new(cookie.value().to_string()))
                .await
            {
                Ok(session) => Ok(ExtractSession(session.session)),
                _ => Ok(ExtractSession(None)),
            };
        }
        Ok(ExtractSession(None))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireSession
where
    S: Send + Sync,
    S: UseUseCase<GetSessionInput, GetSessionOutput>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        for cookie in parts.headers.get_all(COOKIE) {
            let Ok(cookie) = cookie.to_str() else {
                continue
            };
            let Ok(cookie) = Cookie::parse(cookie) else {
                continue
            };
            if cookie.name() != constants::SESSION_COOKIE_ID {
                continue;
            }
            return match state
                .usecase()
                .handle(GetSessionInput::new(cookie.value().to_string()))
                .await
            {
                Ok(GetSessionOutput {
                    session: Some(session),
                }) => Ok(RequireSession(session)),
                _ => Err((StatusCode::BAD_REQUEST, "invalid session")),
            };
        }
        Err((StatusCode::BAD_REQUEST, "invalid session"))
    }
}
