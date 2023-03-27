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
use trait_set::trait_set;

pub struct ExtractSession(pub Option<Session>);
pub struct RequireSession(pub Session);

trait_set! {
    trait State = Send + Sync + UseUseCase<GetSessionInput, GetSessionOutput>;
}

#[async_trait]
impl<S: State> FromRequestParts<S> for ExtractSession {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(ExtractSession(get_session(parts, state).await))
    }
}

#[async_trait]
impl<S: State> FromRequestParts<S> for RequireSession {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match get_session(parts, state).await {
            Some(session) => Ok(RequireSession(session)),
            _ => Err((StatusCode::BAD_REQUEST, "invalid session")),
        }
    }
}

async fn get_session<S: State>(parts: &mut Parts, state: &S) -> Option<Session> {
    for cookie in parts.headers.get_all(COOKIE) {
        let Ok(cookie) = cookie.to_str() else {
            continue
        };
        for cookie in Cookie::split_parse_encoded(cookie) {
            let Ok(cookie) = cookie else {
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
                Ok(output) => output.session,
                _ => None,
            };
        }
    }
    None
}
