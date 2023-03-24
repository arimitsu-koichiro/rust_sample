use crate::{Present, Presenter};
use async_trait::async_trait;

#[derive(Clone, Default)]
pub struct UnitPresenter;

impl Presenter for UnitPresenter {}

#[async_trait]
impl<A: 'static + Send> Present<A> for UnitPresenter {
    type Output = ();

    async fn present(&self, _: A, _: ()) -> Self::Output {}
}
