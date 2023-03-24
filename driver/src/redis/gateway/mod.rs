pub mod pubsub {
    use crate::redis::{RedisPrimaryContext, RedisReaderContext};
    use anyhow::Context as _;
    use futures::StreamExt;
    use kernel::{unexpected, Result};
    use log;
    use redis::AsyncCommands;
    use tokio::sync::mpsc::Receiver;

    pub async fn publish(
        ctx: impl RedisPrimaryContext,
        channel: String,
        message: &[u8],
    ) -> Result<()> {
        let mut conn = ctx.primary().await?;
        conn.publish(channel, message)
            .await
            .with_context(|| unexpected!("publish error"))
    }

    pub async fn subscribe(
        ctx: impl RedisReaderContext,
        channel: String,
    ) -> Result<Receiver<Vec<u8>>> {
        let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(1000);
        let redis = ctx;
        tokio::spawn(async move {
            let mut subscribe_conn = match redis.subscribe_connection().await {
                Ok(sub) => sub.into_pubsub(),
                Err(e) => panic!("{e:?}"),
            };
            let mut subscribe_stream = match subscribe_conn.subscribe(channel).await {
                Ok(_) => subscribe_conn.on_message(),
                Err(e) => panic!("{e:?}"),
            };
            while let Some(sub) = subscribe_stream.next().await {
                match tx.send(sub.get_payload_bytes().to_vec()).await {
                    Ok(_) => (),
                    Err(e) => {
                        log::debug!("send error: {}", e);
                        break;
                    }
                }
            }
        });
        Ok(rx)
    }
}
