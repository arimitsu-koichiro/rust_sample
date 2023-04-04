use crate::Present;
use application::usecase::channel::SubscribeOutput;
use async_trait::async_trait;
use futures::StreamExt;
use kernel::Result;
use tokio::sync::mpsc::Receiver;
use tokio_stream::wrappers::ReceiverStream;

#[derive(Clone, Default)]
pub struct LoggingPresenter;

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
        let mut stream = ReceiverStream::new(rx);
        while let Some(msg) = stream.next().await {
            log::info!(
                "receive message: {}",
                String::from_utf8_lossy(msg.as_slice()).to_string()
            );
        }
    })
    .await;
}
