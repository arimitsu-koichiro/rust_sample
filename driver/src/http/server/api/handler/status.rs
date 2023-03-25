use crate::dispatch;
use crate::http::server::api::{ApiMods, ServerPresenter};

use application::usecase::status::StatusInput;
use axum::extract::State;
use axum::response::Response;
use kernel::Result;

pub(crate) async fn get_status<M: ApiMods<P>, P: ServerPresenter>(
    State(api): State<M>,
) -> Result<Response, ()> {
    dispatch(StatusInput, api).await
}
