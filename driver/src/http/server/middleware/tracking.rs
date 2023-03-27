use crate::http::server::response::constants;
use axum::body::Body;
use axum::headers::HeaderValue;
use axum::http::header::{COOKIE, SET_COOKIE};

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use cookie::{Cookie, CookieBuilder, SameSite};
use futures::future::BoxFuture;
use helper::uuid;
use helper::uuid::ToBase62;
use std::task::{Context, Poll};
use tower::{Layer, Service};

#[derive(Clone)]
pub struct Tracking(pub TrackingData);

#[derive(Clone)]
pub struct TrackingData {
    pub id: String,
}

#[derive(Clone, Default)]
pub struct TrackingLayer;

#[derive(Clone)]
pub struct TrackingMiddleware<S> {
    inner: S,
}

#[async_trait]
impl<S> FromRequestParts<S> for Tracking
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let Some(tracking) = parts.extensions.get::<Tracking>() else {
            return Err((StatusCode::BAD_REQUEST, "failed to handle request"))
        };
        Ok(tracking.clone())
    }
}

impl<S> Layer<S> for TrackingLayer {
    type Service = TrackingMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TrackingMiddleware { inner }
    }
}

impl<S> Service<Request<Body>> for TrackingMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request<Body>) -> Self::Future {
        let tracking = get_tracking_data(&request).unwrap_or_else(|| {
            Tracking(TrackingData {
                id: uuid::new_v4().to_base62(),
            })
        });
        request.extensions_mut().insert(tracking.clone());
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response: Response = future.await?;
            let builder = CookieBuilder::new(constants::TRACKING_COOKIE_ID, tracking.0.id);
            let c = builder
                .secure(true)
                .path("/")
                .max_age(time::Duration::seconds(86400 * 365))
                .same_site(SameSite::Strict)
                .finish();
            response.headers_mut().append(
                SET_COOKIE,
                HeaderValue::from_str(c.to_string().as_str()).unwrap(),
            );
            Ok(response)
        })
    }
}

fn get_tracking_data(request: &Request<Body>) -> Option<Tracking> {
    for cookie in request.headers().get_all(COOKIE) {
        let Ok(cookie) = cookie.to_str() else {
            continue
        };
        for cookie in Cookie::split_parse_encoded(cookie) {
            let Ok(cookie) = cookie else {
                continue
            };
            if cookie.name() != constants::TRACKING_COOKIE_ID {
                continue;
            }

            let data = TrackingData {
                id: cookie.value().to_string(),
            };
            return Some(Tracking(data));
        }
    }
    None
}
