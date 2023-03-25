use crate::dispatch;
use crate::http::server::api::{Mods, ServerPresenter};

use application::usecase::status::StatusInput;
use axum::extract::State;
use axum::response::Response;
use kernel::Result;

pub(crate) async fn get_status<M: Mods<P>, P: ServerPresenter>(
    State(mods): State<M>,
) -> Result<Response, ()> {
    dispatch(StatusInput, mods).await
}
