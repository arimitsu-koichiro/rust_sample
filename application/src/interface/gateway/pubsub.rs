use crate::interface::Component;
use async_trait::async_trait;
use blanket::blanket;
use kernel::Result;
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
