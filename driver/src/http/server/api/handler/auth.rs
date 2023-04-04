use crate::dispatch;
use crate::http::server::api::{Mods, Presenter};
use crate::http::server::middleware::session::{ExtractSession, RequireSession};
use application::usecase::auth::{
    ForgetPasswordInput, GetAuthStatusInput, ResetPasswordInput, SignInInput, SignOutInput,
    SignUpFinishInput, SignUpInput,
};
use axum::extract::{Host, State};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use kernel::Result;
use openapi::models::{
    ForgetPasswordRequest, ResetPasswordRequest, SignUpFinishRequest, SignUpRequest, SigninRequest,
};

pub(crate) fn route<M: Mods<P>, P: Presenter>(_: M) -> Router<M> {
    Router::new()
        .route("/auth/status", get(auth_status::<M, P>))
        .route("/auth/signup", post(signup::<M, P>))
        .route("/auth/signup/finish", post(signup_finish::<M, P>))
        .route("/auth/signin", post(signin::<M, P>))
        .route("/auth/signout", post(signout::<M, P>))
        .route("/auth/forget_password", post(forget_password::<M, P>))
        .route("/auth/reset_password", post(reset_password::<M, P>))
}

async fn auth_status<M: Mods<P>, P: Presenter>(
    State(mods): State<M>,
    ExtractSession(session): ExtractSession,
) -> Result<Response, ()> {
    dispatch(GetAuthStatusInput::new(session), mods).await
}

async fn signup<M: Mods<P>, P: Presenter>(
    State(mods): State<M>,
    Host(host): Host,
    Json(SignUpRequest { mail, password }): Json<SignUpRequest>,
) -> Result<Response, ()> {
    dispatch(SignUpInput::new(mail, password, host), mods).await
}

async fn signup_finish<M: Mods<P>, P: Presenter>(
    State(mods): State<M>,
    Json(SignUpFinishRequest { code }): Json<SignUpFinishRequest>,
) -> Result<Response, ()> {
    dispatch(SignUpFinishInput::new(code), mods).await
}

async fn signin<M: Mods<P>, P: Presenter>(
    State(mods): State<M>,
    Json(SigninRequest {
        mail,
        password,
        remember_me,
    }): Json<SigninRequest>,
) -> Result<Response, ()> {
    dispatch(SignInInput::new(mail, password, remember_me), mods).await
}

async fn signout<M: Mods<P>, P: Presenter>(
    State(mods): State<M>,
    RequireSession(session): RequireSession,
) -> Result<Response, ()> {
    dispatch(SignOutInput::new(session.id), mods).await
}

async fn forget_password<M: Mods<P>, P: Presenter>(
    State(mods): State<M>,
    Host(host): Host,
    Json(ForgetPasswordRequest { mail }): Json<ForgetPasswordRequest>,
) -> Result<Response, ()> {
    dispatch(ForgetPasswordInput::new(mail, host), mods).await
}

async fn reset_password<M: Mods<P>, P: Presenter>(
    State(mods): State<M>,
    Json(ResetPasswordRequest { code, password }): Json<ResetPasswordRequest>,
) -> Result<Response, ()> {
    dispatch(ResetPasswordInput::new(code, password), mods).await
}
