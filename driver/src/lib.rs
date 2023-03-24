use application::usecase::UseCase;
use async_trait::async_trait;
use kernel::Result;

pub mod adapter;

pub mod aws;
pub mod cli;
pub mod http;
pub mod mysql;
pub mod redis;

pub trait Presenter {}

#[async_trait]
pub trait Present<D, Attachment = ()> {
    type Output;
    async fn present(&self, data: D, attachment: Attachment) -> Self::Output;
}

pub trait UsePresenter {
    type Presenter: Presenter;
    fn presenter(&self) -> Self::Presenter;
}

pub(crate) async fn dispatch<U: UseCase<I, O>, I, O, P: Present<Result<O>>>(
    interactor: U,
    input: I,
    presenter: P,
) -> P::Output {
    dispatch_with(interactor, input, presenter, ()).await
}

pub(crate) async fn dispatch_with<U: UseCase<I, O>, I, O, P: Present<Result<O>, A>, A>(
    interactor: U,
    input: I,
    presenter: P,
    attachment: A,
) -> P::Output {
    presenter
        .present(interactor.handle(input).await, attachment)
        .await
}
