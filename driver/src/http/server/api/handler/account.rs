use crate::dispatch;
use crate::http::server::api::{Mods, Presenter};
use crate::http::server::middleware::session::RequireSession;
use application::usecase::account::GetAccountInput;
use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use kernel::Result;

pub(crate) fn route<M: Mods<P>, P: Presenter>(_: M) -> Router<M> {
    Router::new().route("/account/:id", get(get_account::<M, P>))
}

async fn get_account<M: Mods<P>, P: Presenter>(
    State(mods): State<M>,
    Path(id): Path<String>,
    RequireSession(session): RequireSession,
) -> Result<Response, ()> {
    dispatch(GetAccountInput::new(id, Some(session.id)), mods).await
}
