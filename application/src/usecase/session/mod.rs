use crate::interface::repository::session::UseSessionRepository;
use crate::interface::{Component, UseContext};
use crate::internal;
use crate::usecase::UseCase;
use async_trait::async_trait;
use kernel::entity::Session;
use kernel::Result;
use std::marker::PhantomData;
use trait_set::trait_set;

#[derive(Clone, new)]
pub struct GetSessionUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait GetSessionUseCaseDeps<C: Component> = Component
    + UseContext<Context = C>
    + UseSessionRepository<C>
    ;
}

#[async_trait]
impl<C, Deps> UseCase<GetSessionInput, GetSessionOutput> for GetSessionUseCase<C, Deps>
where
    C: Component,
    Deps: GetSessionUseCaseDeps<C>,
{
    async fn handle(&self, input: GetSessionInput) -> Result<GetSessionOutput> {
        let session = internal::session::get_session(
            self.deps.context().await?,
            self.deps.session_repository(),
            input.session_id,
        )
        .await?;
        Ok(GetSessionOutput::new(session))
    }
}

#[derive(new)]
pub struct GetSessionInput {
    session_id: String,
}

#[derive(new)]
pub struct GetSessionOutput {
    pub session: Option<Session>,
}
