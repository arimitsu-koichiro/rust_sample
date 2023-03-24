use crate::{Present, Presenter};
use application::usecase::channel::PubSubOutput;
use async_trait::async_trait;
use futures::StreamExt;
use kernel::Result;

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
        let _ = tokio::spawn(async move {
            let mut stream = tokio_stream::wrappers::ReceiverStream::new(output.rx);
            while let Some(msg) = stream.next().await {
                log::info!(
                    "receive message: {}",
                    String::from_utf8_lossy(msg.as_slice()).to_string()
                );
            }
        })
        .await;
    }
}
