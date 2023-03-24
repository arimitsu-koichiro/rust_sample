use crate::dispatch;
use crate::http::server::api::{ApiMods, ServerPresenter};
use crate::http::server::middleware::session::RequireSession;
use application::interface::Component;
use application::usecase::account::{GetAccountInput, GetAccountUseCase};
use axum::extract::{Path, State};
use axum::response::Response;
use kernel::Result;

pub(crate) async fn get_account<M: ApiMods<C, P>, C: Component, P: ServerPresenter>(
    State(api): State<M>,
    Path(id): Path<String>,
    RequireSession(session): RequireSession,
) -> Result<Response, ()> {
    dispatch(
        GetAccountUseCase::new(api.clone()),
        GetAccountInput::new(id, Some(session.id)),
        api.presenter(),
    )
    .await
}
