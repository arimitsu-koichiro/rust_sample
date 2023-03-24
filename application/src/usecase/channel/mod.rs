use crate::interface::gateway::pubsub::{PubSubGateway, UsePubSubGateway};
use crate::interface::Component;
use crate::interface::UseContext;
use crate::usecase::UseCase;
use anyhow::Result;
use async_trait::async_trait;
use log;
use std::marker::PhantomData;
use tokio::sync::mpsc::Receiver;
use tokio_stream::StreamExt;
use trait_set::trait_set;

#[derive(Clone, new)]
pub struct SubscribeUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait SubscribeUseCaseDeps<C: Component> = Component + UseContext<Context = C>
    + UsePubSubGateway<C>
    ;
}
#[async_trait]
impl<C, Deps> UseCase<SubscribeInput, SubscribeOutput> for SubscribeUseCase<C, Deps>
where
    C: Component,
    Deps: SubscribeUseCaseDeps<C>,
{
    async fn handle(&self, input: SubscribeInput) -> Result<SubscribeOutput> {
        let rx = self
            .deps
            .pubsub_gateway()
            .subscribe(
                self.deps.context().await?,
                format!("channel:{}", input.channel_id),
            )
            .await?;
        Ok(SubscribeOutput::new(rx))
    }
}

#[derive(Clone, new)]
pub struct PublishUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait PublishUseCaseDeps<C: Component> = Component + UseContext<Context = C>
    + UsePubSubGateway<C>
    ;
}
#[async_trait]
impl<C, Deps> UseCase<PublishInput, PublishOutput> for PublishUseCase<C, Deps>
where
    C: Component,
    Deps: PublishUseCaseDeps<C>,
{
    async fn handle(&self, input: PublishInput) -> Result<PublishOutput> {
        self.deps
            .pubsub_gateway()
            .publish(
                self.deps.context().await?,
                format!("channel:{}", input.channel_id),
                input.message.as_slice(),
            )
            .await?;
        Ok(PublishOutput)
    }
}

#[derive(Clone, new)]
pub struct PubSubUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait PubSubUseCaseDeps<C: Component> = Component + UseContext<Context = C>
    + UsePubSubGateway<C>
    ;
}

#[async_trait]
impl<C, Deps> UseCase<PubSubInput, PubSubOutput> for PubSubUseCase<C, Deps>
where
    C: Component,
    Deps: PubSubUseCaseDeps<C>,
{
    async fn handle(&self, input: PubSubInput) -> Result<PubSubOutput> {
        let receiver = input.receiver;
        let (sender, rx) = tokio::sync::mpsc::channel(1000);
        let channel_id = input.channel_id.clone();
        let ctx = self.deps.context().await?;
        let _ctx = ctx.clone();
        let gateway = self.deps.pubsub_gateway();
        let mut publish_task = tokio::spawn(async move {
            let mut stream = tokio_stream::wrappers::ReceiverStream::new(receiver);
            while let Some(message) = stream.next().await {
                if message.is_empty() {
                    continue;
                }
                match gateway
                    .publish(
                        _ctx.clone(),
                        format!("channel:{channel_id}"),
                        message.as_slice(),
                    )
                    .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        log::debug!("publish error: {}", e);
                        break;
                    }
                }
            }
        });
        let receiver = self
            .deps
            .pubsub_gateway()
            .subscribe(ctx.clone(), format!("channel:{}", input.channel_id))
            .await?;
        let mut subscribe_task = tokio::spawn(async move {
            let mut stream = tokio_stream::wrappers::ReceiverStream::new(receiver);
            while let Some(msg) = stream.next().await {
                match sender.send(msg).await {
                    Ok(_) => (),
                    Err(e) => {
                        log::debug!("send error: {}", e);
                        break;
                    }
                }
            }
        });
        tokio::spawn(async move {
            tokio::select! {
                val = (&mut publish_task) => {
                    match val {
                        Ok(r) => log::debug!("publish_task finish: {:?}", r),
                        Err(e) => log::warn!("publish_task error: {}", e)
                    }
                    subscribe_task.abort();
                }
                val = (&mut subscribe_task) => {
                    match val {
                        Ok(r) => log::debug!("subscribe_task finish: {:?}", r),
                        Err(e) => log::warn!("subscribe_task error: {}", e)
                    }
                    publish_task.abort();
                }
            }
        });
        Ok(PubSubOutput { rx })
    }
}

#[derive(new)]
pub struct PublishInput {
    pub(crate) channel_id: String,
    pub(crate) message: Vec<u8>,
}

#[derive(new)]
pub struct PublishOutput;

#[derive(new)]
pub struct SubscribeInput {
    pub(crate) channel_id: String,
}

#[derive(new)]
pub struct SubscribeOutput {
    pub rx: Receiver<Vec<u8>>,
}

#[derive(new)]
pub struct PubSubInput {
    pub(crate) channel_id: String,
    pub(crate) receiver: Receiver<Vec<u8>>,
}

#[derive(new)]
pub struct PubSubOutput {
    pub rx: Receiver<Vec<u8>>,
}
