use crate::dispatch;
use crate::http::server::api::{Mods, ServerPresenter};
use crate::http::server::middleware::session::RequireSession;

use application::usecase::account::GetAccountInput;
use axum::extract::{Path, State};
use axum::response::Response;
use kernel::Result;

pub(crate) async fn get_account<M: Mods<P>, P: ServerPresenter>(
    State(mods): State<M>,
    Path(id): Path<String>,
    RequireSession(session): RequireSession,
) -> Result<Response, ()> {
    dispatch(GetAccountInput::new(id, Some(session.id)), mods).await
}
