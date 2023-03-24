use super::handler::{account, auth, channel, status};
use crate::http::server::api::{ApiMods, ServerPresenter};
use crate::http::server::middleware::csrf::csrf_protection;
use crate::http::server::middleware::request_id::MakeRequestBase62Uuid;
use crate::http::server::middleware::session::RequireSession;
use application::interface::repository::Transaction;
use axum::middleware::from_extractor_with_state;
use axum::routing::{get, post, Router};
use tower_http::request_id::{PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

pub(crate) fn define_route<M: ApiMods<C, P>, C: Transaction, P: ServerPresenter>(
    mods: M,
) -> Router {
    let private = || from_extractor_with_state::<RequireSession, M>(mods.clone());
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/status", get(status::get_status::<M, C, P>))
                .route(
                    "/account/:id",
                    get(account::get_account::<M, C, P>).route_layer(private()),
                )
                .route("/auth/status", get(auth::auth_status::<M, C, P>))
                .route("/auth/signup", post(auth::signup::<M, C, P>))
                .route("/auth/signup/finish", post(auth::signup_finish::<M, C, P>))
                .route("/auth/signin", post(auth::signin::<M, C, P>))
                .route(
                    "/auth/signout",
                    post(auth::signout::<M, C, P>).route_layer(private()),
                )
                .route(
                    "/auth/forget_password",
                    post(auth::forget_password::<M, C, P>),
                )
                .route(
                    "/auth/reset_password",
                    post(auth::reset_password::<M, C, P>),
                )
                .route(
                    "/channel/:channel_id",
                    get(channel::subscribe_channel::<M, C, P>).route_layer(private()),
                )
                .route(
                    "/channel/:channel_id",
                    post(channel::publish_channel::<M, C, P>).route_layer(private()),
                )
                .route(
                    "/channel/:channel_id/socket",
                    get(channel::channel_socket::<M, C, P>).route_layer(private()),
                ),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestBase62Uuid))
        .layer(axum::middleware::from_fn(csrf_protection))
        .with_state(mods)
}
