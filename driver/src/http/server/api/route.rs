use super::handler::{account, auth, channel, status};
use crate::http::server::api::{Mods, Presenter};
use crate::http::server::middleware::csrf::csrf_protection;
use crate::http::server::middleware::request_id::MakeRequestBase62Uuid;

use crate::http::server::middleware::tracking::TrackingLayer;
use axum::middleware::from_fn;
use axum::routing::Router;
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

pub(crate) fn define_route<M: Mods<P>, P: Presenter>(mods: M) -> Router {
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .merge(status::route(mods.clone()))
                .merge(account::route(mods.clone()))
                .merge(auth::route(mods.clone()))
                .merge(channel::route(mods.clone())), // .route("/status", get(status::get_status::<M, P>))
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestBase62Uuid))
        .layer(from_fn(csrf_protection))
        .layer(TrackingLayer::default())
        .with_state(mods)
}
