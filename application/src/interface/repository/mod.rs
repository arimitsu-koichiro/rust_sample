pub mod account;
pub mod authentication;
pub mod comment;
pub mod session;

use crate::interface::Component;
use async_trait::async_trait;
use kernel::Result;

#[async_trait]
pub trait Transaction: Component + Sized {
    async fn begin(&self) -> Result<Self>;
    async fn commit(&self) -> Result<Self>;
}

#[async_trait]
impl Transaction for () {
    async fn begin(&self) -> Result<()> {
        Ok(())
    }
    async fn commit(&self) -> Result<()> {
        Ok(())
    }
}
