use crate::interface::Component;
use async_trait::async_trait;
use blanket::blanket;
use kernel::Result;

pub mod account;
pub mod auth;
pub mod channel;
pub mod status;

#[async_trait]
#[blanket(derive(Arc))]
pub trait UseCase<I, O>: Component {
    async fn handle(&self, input: I) -> Result<O>;
}
