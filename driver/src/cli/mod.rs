use crate::dispatch;
use crate::{Present, UsePresenter};
use application::interface::Component;
use application::usecase::channel::{SubscribeInput, SubscribeOutput};
use application::usecase::UseUseCase;
use clap::Parser;
use kernel::Result;
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
    let _ = dispatch(SubscribeInput::new(args.channel), mods).await;
    Ok(())
}

trait_set! {
    pub trait Mods<P: CliPresenter> = Component
    + UseUseCase<SubscribeInput, SubscribeOutput>
    + UsePresenter<Presenter = P>
    ;
    pub trait CliPresenter = Component
    + Present<Result<SubscribeOutput>>
    ;
}
