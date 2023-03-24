use crate::http::server::response::constants;
use application::interface::repository::session::{SessionRepository, UseSessionRepository};
use application::interface::Component;
use application::interface::UseContext;
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
impl<S, C> FromRequestParts<S> for ExtractSession
where
    C: Component,
    S: Send + Sync,
    S: UseContext<Context = C>,
    S: UseSessionRepository<C>,
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
                .session_repository()
                .get(state.context().await.unwrap(), cookie.value().to_string())
                .await
            {
                Ok(session) => Ok(ExtractSession(session)),
                _ => Ok(ExtractSession(None)),
            };
        }
        Ok(ExtractSession(None))
    }
}

#[async_trait]
impl<S, C> FromRequestParts<S> for RequireSession
where
    C: Component,
    S: Send + Sync,
    S: UseContext<Context = C>,
    S: UseSessionRepository<C>,
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
            if let Ok(Some(session)) = state
                .session_repository()
                .get(state.context().await.unwrap(), cookie.value().to_string())
                .await
            {
                return Ok(RequireSession(session));
            }
            return match state
                .session_repository()
                .get(state.context().await.unwrap(), cookie.value().to_string())
                .await
            {
                Ok(Some(session)) => Ok(RequireSession(session)),
                _ => Err((StatusCode::BAD_REQUEST, "invalid session")),
            };
        }
        Err((StatusCode::BAD_REQUEST, "invalid session"))
    }
}
