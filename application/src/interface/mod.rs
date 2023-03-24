use async_trait::async_trait;
use kernel::Result;
use trait_set::trait_set;

pub mod gateway;
pub mod repository;

trait_set! {
    /// marker trait for DI
    pub trait Component = 'static + Clone + Send + Sync;
}

#[async_trait]
pub trait UseContext {
    type Context;
    async fn context(&self) -> Result<Self::Context>;
}
