use application::usecase::{UseCase, UseUseCase};
use async_trait::async_trait;
use kernel::Result;

pub mod adapter;

pub mod aws;
pub mod cli;
pub mod http;
pub mod mysql;
pub mod redis;

#[async_trait]
pub trait Present<D, Attachment = ()> {
    type Output;
    async fn present(&self, data: D, attachment: Attachment) -> Self::Output;
}

pub trait UsePresenter {
    type Presenter;
    fn presenter(&self) -> Self::Presenter;
}

pub(crate) async fn dispatch<
    M: UseUseCase<I, O> + UsePresenter<Presenter = P>,
    I,
    O,
    P: Present<Result<O>>,
>(
    input: I,
    mods: M,
) -> P::Output {
    dispatch_with(input, (), mods).await
}

pub(crate) async fn dispatch_with<
    M: UseUseCase<I, O> + UsePresenter<Presenter = P>,
    I,
    O,
    P: Present<Result<O>, A>,
    A,
>(
    input: I,
    attachment: A,
    mods: M,
) -> P::Output {
    mods.presenter()
        .present(mods.usecase().handle(input).await, attachment)
        .await
}
