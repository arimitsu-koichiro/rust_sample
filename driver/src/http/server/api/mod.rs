use crate::http::server::api::config::ApiConfig;
use crate::http::server::api::route::define_route;
use crate::{Present, UsePresenter};
use anyhow::Context as _;
use application::interface::Component;
use application::usecase::account::{GetAccountInput, GetAccountOutput};
use application::usecase::auth::{
    ForgetPasswordInput, ForgetPasswordOutput, GetAuthStatusInput, GetAuthStatusOutput,
    ResetPasswordInput, ResetPasswordOutput, SignInInput, SignInOutput, SignOutInput,
    SignOutOutput, SignUpFinishInput, SignUpFinishOutput, SignUpInput, SignUpOutput,
};
use application::usecase::channel::{
    PubSubInput, PubSubOutput, PublishInput, PublishOutput, SubscribeInput, SubscribeOutput,
};
use application::usecase::session::{GetSessionInput, GetSessionOutput};
use application::usecase::status::{StatusInput, StatusOutput};
use application::usecase::UseUseCase;
use axum::response::Response;
use axum::Server;
use kernel::{unexpected, Result};
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;
use trait_set::trait_set;

pub mod config;
mod handler;
pub mod presenter;
mod route;

pub async fn start<M: ApiMods<P>, P: ServerPresenter>(config: ApiConfig, mods: M) -> Result<()> {
    let app = define_route(mods);
    Server::bind(&config.bind_address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .with_context(|| unexpected!("server error"))
}

trait_set! {
    pub trait ApiUseCases =
    Component
    + UseUseCase<StatusInput, StatusOutput>
    + UseUseCase<GetAccountInput, GetAccountOutput>
    + UseUseCase<GetAuthStatusInput, GetAuthStatusOutput>
    + UseUseCase<SignUpInput, SignUpOutput>
    + UseUseCase<SignUpFinishInput, SignUpFinishOutput>
    + UseUseCase<SignInInput, SignInOutput>
    + UseUseCase<SignOutInput, SignOutOutput>
    + UseUseCase<ForgetPasswordInput, ForgetPasswordOutput>
    + UseUseCase<ResetPasswordInput, ResetPasswordOutput>
    + UseUseCase<PublishInput, PublishOutput>
    + UseUseCase<SubscribeInput, SubscribeOutput>
    + UseUseCase<PubSubInput, PubSubOutput>
    + UseUseCase<GetSessionInput, GetSessionOutput>
    ;
    pub trait ApiMods<P: ServerPresenter> = Component
    + ApiUseCases
    + UsePresenter<Presenter = P>
    ;
    pub trait PresentResponse<D> = Present<Result<D>, Output = Result<Response, ()>>;
    pub trait ServerPresenter = Component
    + PresentResponse<StatusOutput>
    + PresentResponse<GetAccountOutput>
    + PresentResponse<GetAuthStatusOutput>
    + PresentResponse<SignUpOutput>
    + PresentResponse<SignUpFinishOutput>
    + PresentResponse<SignInOutput>
    + PresentResponse<SignOutOutput>
    + PresentResponse<ForgetPasswordOutput>
    + PresentResponse<ResetPasswordOutput>
    + PresentResponse<PublishOutput>
    + PresentResponse<SubscribeOutput>
    + Present<Result<PubSubOutput>, Sender<Vec<u8>>, Output=()>
    ;
}
