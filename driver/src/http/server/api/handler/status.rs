use crate::dispatch;
use crate::http::server::api::{Mods, Presenter};

use application::usecase::status::StatusInput;
use axum::extract::State;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use kernel::Result;

pub(crate) fn route<M: Mods<P>, P: Presenter>(_: M) -> Router<M> {
    Router::new().route("/status", get(get_status::<M, P>))
}

async fn get_status<M: Mods<P>, P: Presenter>(State(mods): State<M>) -> Result<Response, ()> {
    dispatch(StatusInput, mods).await
}
