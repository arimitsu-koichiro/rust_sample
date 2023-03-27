use crate::dispatch;
use crate::http::server::api::{Mods, ServerPresenter};
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

pub(crate) async fn auth_status<M: Mods<P>, P: ServerPresenter>(
    State(mods): State<M>,
    ExtractSession(session): ExtractSession,
) -> Result<Response, ()> {
    dispatch(GetAuthStatusInput::new(session), mods).await
}

pub(crate) async fn signup<M: Mods<P>, P: ServerPresenter>(
    State(mods): State<M>,
    Host(site_url): Host,
    Json(SignUpRequest { mail, password }): Json<SignUpRequest>,
) -> Result<Response, ()> {
    dispatch(SignUpInput::new(mail, password, site_url), mods).await
}

pub(crate) async fn signup_finish<M: Mods<P>, P: ServerPresenter>(
    State(mods): State<M>,
    Query(SignUpFinishRequest { code }): Query<SignUpFinishRequest>,
) -> Result<Response, ()> {
    dispatch(SignUpFinishInput::new(code), mods).await
}

pub(crate) async fn signin<M: Mods<P>, P: ServerPresenter>(
    State(mods): State<M>,
    Json(SigninRequest {
        mail,
        password,
        remember_me,
    }): Json<SigninRequest>,
) -> Result<Response, ()> {
    dispatch(SignInInput::new(mail, password, remember_me), mods).await
}

pub(crate) async fn signout<M: Mods<P>, P: ServerPresenter>(
    State(mods): State<M>,
    RequireSession(session): RequireSession,
) -> Result<Response, ()> {
    dispatch(SignOutInput::new(session.id), mods).await
}

pub(crate) async fn forget_password<M: Mods<P>, P: ServerPresenter>(
    State(mods): State<M>,
    Host(site_url): Host,
    Json(ForgetPasswordRequest { mail }): Json<ForgetPasswordRequest>,
) -> Result<Response, ()> {
    dispatch(ForgetPasswordInput::new(mail, site_url), mods).await
}

pub(crate) async fn reset_password<M: Mods<P>, P: ServerPresenter>(
    State(mods): State<M>,
    Json(ResetPasswordRequest { code, password }): Json<ResetPasswordRequest>,
) -> Result<Response, ()> {
    dispatch(ResetPasswordInput::new(code, password), mods).await
}
