use crate::dispatch;
use crate::http::server::api::{ApiMods, ServerPresenter};
use application::interface::Component;
use application::usecase::status::{StatusInput, StatusUseCase};
use axum::extract::State;
use axum::response::Response;
use kernel::Result;

pub(crate) async fn get_status<M: ApiMods<C, P>, C: Component, P: ServerPresenter>(
    State(api): State<M>,
) -> Result<Response, ()> {
    dispatch(
        StatusUseCase::new(api.clone()),
        StatusInput,
        api.presenter(),
    )
    .await
}
