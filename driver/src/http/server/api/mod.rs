use crate::http::server::api::config::ApiConfig;
use crate::http::server::api::route::define_route;
use crate::{Present, UsePresenter};
use anyhow::Context as _;
use application::interface::gateway::mail::UseMailGateway;
use application::interface::gateway::pubsub::UsePubSubGateway;
use application::interface::repository::account::UseAccountRepository;
use application::interface::repository::authentication::UseAuthenticationRepository;
use application::interface::repository::comment::UseCommentRepository;
use application::interface::repository::session::UseSessionRepository;
use application::interface::repository::Transaction;
use application::interface::{Component, UseContext};
use application::usecase::account::GetAccountOutput;
use application::usecase::auth::{
    ForgetPasswordOutput, GetAuthStatusOutput, ResetPasswordOutput, SignInOutput, SignOutOutput,
    SignUpFinishOutput, SignUpOutput,
};
use application::usecase::channel::{PubSubOutput, PublishOutput, SubscribeOutput};
use application::usecase::status::StatusOutput;
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

pub async fn start<C: Transaction, M: ApiMods<C, P>, P: ServerPresenter>(
    config: ApiConfig,
    mods: M,
) -> Result<()> {
    let app = define_route(mods);
    Server::bind(&config.bind_address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .with_context(|| unexpected!("server error"))
}

trait_set! {
    pub trait ApiMods<C: Component, P: ServerPresenter> = Component
    + UsePresenter<Presenter = P>
    + UseContext<Context = C>
    + UseAccountRepository<C>
    + UseSessionRepository<C>
    + UseAuthenticationRepository<C>
    + UseCommentRepository<C>
    + UseMailGateway<C>
    + UsePubSubGateway<C>
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
