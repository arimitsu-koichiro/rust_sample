use crate::dispatch;
use crate::http::server::api::{ApiMods, ServerPresenter};
use crate::http::server::middleware::session::{ExtractSession, RequireSession};

use application::usecase::auth::{
    ForgetPasswordInput, GetAuthStatusInput, ResetPasswordInput, SignInInput, SignOutInput,
    SignUpFinishInput, SignUpInput,
};
use axum::extract::{Host, Query, State};
use axum::response::Response;
use axum::Json;
use kernel::Result;
use openapi::models::{
    ForgetPasswordRequest, ResetPasswordRequest, SignUpFinishRequest, SignUpRequest, SigninRequest,
};

pub(crate) async fn auth_status<M: ApiMods<P>, P: ServerPresenter>(
    State(api): State<M>,
    ExtractSession(session): ExtractSession,
) -> Result<Response, ()> {
    dispatch(GetAuthStatusInput::new(session), api).await
}

pub(crate) async fn signup<M: ApiMods<P>, P: ServerPresenter>(
    State(api): State<M>,
    Host(site_url): Host,
    Json(SignUpRequest { mail, password }): Json<SignUpRequest>,
) -> Result<Response, ()> {
    dispatch(SignUpInput::new(mail, password, site_url), api).await
}

pub(crate) async fn signup_finish<M: ApiMods<P>, P: ServerPresenter>(
    State(api): State<M>,
    Query(SignUpFinishRequest { code }): Query<SignUpFinishRequest>,
) -> Result<Response, ()> {
    dispatch(SignUpFinishInput::new(code), api).await
}

pub(crate) async fn signin<M: ApiMods<P>, P: ServerPresenter>(
    State(api): State<M>,
    Json(SigninRequest {
        mail,
        password,
        remember_me,
    }): Json<SigninRequest>,
) -> Result<Response, ()> {
    let input = SignInInput::new(mail, password, remember_me.unwrap_or(false));
    dispatch(input, api).await
}

pub(crate) async fn signout<M: ApiMods<P>, P: ServerPresenter>(
    State(api): State<M>,
    RequireSession(session): RequireSession,
) -> Result<Response, ()> {
    dispatch(SignOutInput::new(session.id), api).await
}

pub(crate) async fn forget_password<M: ApiMods<P>, P: ServerPresenter>(
    State(api): State<M>,
    Host(site_url): Host,
    Json(ForgetPasswordRequest { mail }): Json<ForgetPasswordRequest>,
) -> Result<Response, ()> {
    dispatch(ForgetPasswordInput::new(mail, site_url), api).await
}

pub(crate) async fn reset_password<M: ApiMods<P>, P: ServerPresenter>(
    State(api): State<M>,
    Json(ResetPasswordRequest { code, password }): Json<ResetPasswordRequest>,
) -> Result<Response, ()> {
    dispatch(ResetPasswordInput::new(code, password), api).await
}
