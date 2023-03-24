use crate::dispatch;
use crate::http::server::api::{ApiMods, ServerPresenter};
use crate::http::server::middleware::session::{ExtractSession, RequireSession};
use application::interface::repository::Transaction;
use application::interface::Component;
use application::usecase::auth::{
    ForgetPasswordInput, ForgetPasswordUseCase, GetAuthStatusInput, GetAuthStatusUseCase,
    ResetPasswordInput, ResetPasswordUseCase, SignInInput, SignInUseCase, SignOutInput,
    SignOutUseCase, SignUpFinishInput, SignUpInput, SignupFinishUseCase, SignupUseCase,
};
use axum::extract::{Host, Query, State};
use axum::response::Response;
use axum::Json;
use kernel::Result;
use openapi::models::{
    ForgetPasswordRequest, ResetPasswordRequest, SignUpFinishRequest, SignUpRequest, SigninRequest,
};

pub(crate) async fn auth_status<M: ApiMods<C, P>, C: Component, P: ServerPresenter>(
    State(api): State<M>,
    ExtractSession(session): ExtractSession,
) -> Result<Response, ()> {
    dispatch(
        GetAuthStatusUseCase::new(api.clone()),
        GetAuthStatusInput::new(session),
        api.presenter(),
    )
    .await
}

pub(crate) async fn signup<M: ApiMods<C, P>, C: Component, P: ServerPresenter>(
    State(api): State<M>,
    Host(site_url): Host,
    Json(SignUpRequest { mail, password }): Json<SignUpRequest>,
) -> Result<Response, ()> {
    dispatch(
        SignupUseCase::new(api.clone()),
        SignUpInput::new(mail, password, site_url),
        api.presenter(),
    )
    .await
}

pub(crate) async fn signup_finish<M: ApiMods<C, P>, C: Transaction, P: ServerPresenter>(
    State(api): State<M>,
    Query(SignUpFinishRequest { code }): Query<SignUpFinishRequest>,
) -> Result<Response, ()> {
    dispatch(
        SignupFinishUseCase::new(api.clone()),
        SignUpFinishInput::new(code),
        api.presenter(),
    )
    .await
}

pub(crate) async fn signin<M: ApiMods<C, P>, C: Component, P: ServerPresenter>(
    State(api): State<M>,
    Json(SigninRequest {
        mail,
        password,
        remember_me,
    }): Json<SigninRequest>,
) -> Result<Response, ()> {
    dispatch(
        SignInUseCase::new(api.clone()),
        SignInInput::new(mail, password, remember_me.unwrap_or(false)),
        api.presenter(),
    )
    .await
}

pub(crate) async fn signout<M: ApiMods<C, P>, C: Component, P: ServerPresenter>(
    State(api): State<M>,
    RequireSession(session): RequireSession,
) -> Result<Response, ()> {
    dispatch(
        SignOutUseCase::new(api.clone()),
        SignOutInput::new(session.id),
        api.presenter(),
    )
    .await
}

pub(crate) async fn forget_password<M: ApiMods<C, P>, C: Component, P: ServerPresenter>(
    State(api): State<M>,
    Host(site_url): Host,
    Json(ForgetPasswordRequest { mail }): Json<ForgetPasswordRequest>,
) -> Result<Response, ()> {
    dispatch(
        ForgetPasswordUseCase::new(api.clone()),
        ForgetPasswordInput::new(mail, site_url),
        api.presenter(),
    )
    .await
}

pub(crate) async fn reset_password<M: ApiMods<C, P>, C: Transaction, P: ServerPresenter>(
    State(api): State<M>,
    Json(ResetPasswordRequest { code, password }): Json<ResetPasswordRequest>,
) -> Result<Response, ()> {
    dispatch(
        ResetPasswordUseCase::new(api.clone()),
        ResetPasswordInput::new(code, password),
        api.presenter(),
    )
    .await
}
