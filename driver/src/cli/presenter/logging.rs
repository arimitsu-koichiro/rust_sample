use crate::{Present, Presenter};
use application::usecase::channel::{PubSubOutput, SubscribeOutput};
use async_trait::async_trait;
use futures::StreamExt;
use kernel::Result;
use tokio::sync::mpsc::Receiver;

#[derive(Clone, Default)]
pub struct LoggingPresenter;

impl Presenter for LoggingPresenter {}

#[async_trait]
impl Present<Result<PubSubOutput>> for LoggingPresenter {
    type Output = ();

    async fn present(&self, data: Result<PubSubOutput>, _: ()) -> Self::Output {
        let Ok(output) = data else {
            return;
        };
        logging_receiver(output.rx).await;
    }
}

#[async_trait]
impl Present<Result<SubscribeOutput>> for LoggingPresenter {
    type Output = ();

    async fn present(&self, data: Result<SubscribeOutput>, _: ()) -> Self::Output {
        let Ok(output) = data else {
            return;
        };
        logging_receiver(output.rx).await;
    }
}

async fn logging_receiver(rx: Receiver<Vec<u8>>) {
    let _ = tokio::spawn(async move {
        let mut stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        while let Some(msg) = stream.next().await {
            log::info!(
                "receive message: {}",
                String::from_utf8_lossy(msg.as_slice()).to_string()
            );
        }
    })
    .await;
}
