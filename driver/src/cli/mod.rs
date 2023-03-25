use crate::dispatch;
use crate::{Present, UsePresenter};
use application::interface::Component;
use application::usecase::channel::{PubSubInput, PubSubOutput};
use clap::Parser;

use kernel::Result;

use application::usecase::UseUseCase;
use tokio::sync::mpsc::channel;
use trait_set::trait_set;

pub mod presenter;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(short, long)]
    channel: String,
}
pub async fn listen_pubsub<M: Mods<P>, P: CliPresenter>(mods: M) -> Result<()> {
    let args = Args::parse();
    let (_exchange_sender, receiver) = channel::<Vec<u8>>(1000);
    let _ = dispatch(PubSubInput::new(args.channel, receiver), mods).await;
    Ok(())
}

trait_set! {
    pub trait Mods<P: CliPresenter> = Component
    + UseUseCase<PubSubInput, PubSubOutput>
    + UsePresenter<Presenter = P>
    ;
    pub trait CliPresenter = Component
    + Present<Result<PubSubOutput>>
    ;
}
