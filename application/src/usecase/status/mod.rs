use crate::interface::Component;
use crate::interface::UseContext;

use kernel::Result;

use crate::usecase::UseCase;
use async_trait::async_trait;
use std::marker::PhantomData;
use trait_set::trait_set;

#[derive(Clone, new)]
pub struct StatusUseCase<C, Deps> {
    _deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait StatusUseCaseDeps<C: Component> = Component + UseContext<Context = C>
    ;
}

#[async_trait]
impl<C, Deps> UseCase<StatusInput, StatusOutput> for StatusUseCase<C, Deps>
where
    C: Component,
    Deps: StatusUseCaseDeps<C>,
{
    async fn handle(&self, _: StatusInput) -> Result<StatusOutput> {
        let version = kernel::build_info::git_commit_hash();
        let build_timestamp = kernel::build_info::build_time_utc();
        Ok(StatusOutput::new(version, build_timestamp))
    }
}

pub struct StatusInput;

#[derive(new)]
pub struct StatusOutput {
    pub version: Option<String>,
    pub build_timestamp: String,
}
