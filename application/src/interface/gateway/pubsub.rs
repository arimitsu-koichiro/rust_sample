use crate::interface::Component;
use async_trait::async_trait;
use blanket::blanket;
use kernel::Result;
#[cfg(test)]
use mockall::mock;
use tokio::sync::mpsc::Receiver;

#[async_trait]
#[blanket(derive(Arc))]
pub trait PubSubGateway<Context>: Component {
    async fn publish(&self, ctx: Context, channel: String, message: &[u8]) -> Result<()>;
    async fn subscribe(&self, ctx: Context, channel: String) -> Result<Receiver<Vec<u8>>>;
}

pub trait UsePubSubGateway<Context> {
    type Gateway: PubSubGateway<Context>;
    fn pubsub_gateway(&self) -> Self::Gateway;
}

#[cfg(test)]
mock! {
    pub PubSubGateway{}
    impl Clone for PubSubGateway {
        fn clone(&self) -> Self;
    }
    #[async_trait]
    impl PubSubGateway<()> for PubSubGateway {
        async fn publish(&self, ctx: (), channel: String, message: &[u8]) -> Result<()>;
        async fn subscribe(&self, ctx: (), channel: String) -> Result<Receiver<Vec<u8>>>;
    }
}
