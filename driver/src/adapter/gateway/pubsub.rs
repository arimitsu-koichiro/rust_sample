use crate::redis::RedisContext;

use application::interface::gateway::pubsub::PubSubGateway;

use async_trait::async_trait;
use derive_new::new;

use kernel::Result;

use tokio::sync::mpsc::Receiver;

#[derive(Clone, Debug, new)]
pub struct PubSubGatewayImpl;

#[async_trait]
impl<Context: RedisContext> PubSubGateway<Context> for PubSubGatewayImpl {
    async fn publish(&self, ctx: Context, channel: String, message: &[u8]) -> Result<()> {
        crate::redis::gateway::pubsub::publish(ctx, channel, message).await
    }

    async fn subscribe(&self, ctx: Context, channel: String) -> Result<Receiver<Vec<u8>>> {
        crate::redis::gateway::pubsub::subscribe(ctx, channel).await
    }
}
