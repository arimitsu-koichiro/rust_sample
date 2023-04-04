use crate::redis::RedisContext;
use application::interface::repository::session::SessionRepository;
use async_trait::async_trait;
use derive_new::new;
use helper::validation::Validation;
use kernel::entity::{ProvisionalSession, Session};
use kernel::Result;

#[derive(Clone, Debug, new)]
pub struct SessionRepositoryImpl;

#[async_trait]
impl<Context> SessionRepository<Context> for SessionRepositoryImpl
where
    Context: RedisContext,
{
    async fn set_provisional_session(
        &self,
        ctx: Context,
        session: ProvisionalSession,
    ) -> Result<()> {
        crate::redis::repository::session::set_provisional_session(ctx, session.validate()?).await
    }
    async fn get_provisional_session(
        &self,
        ctx: Context,
        id: String,
    ) -> Result<Option<ProvisionalSession>> {
        crate::redis::repository::session::get_provisional_session(ctx, id).await
    }
    async fn set(&self, ctx: Context, session: Session) -> Result<()> {
        crate::redis::repository::session::set(ctx, session.validate()?).await
    }

    async fn get(&self, ctx: Context, id: String) -> Result<Option<Session>> {
        crate::redis::repository::session::get(ctx, id).await
    }
    async fn delete(&self, ctx: Context, id: String) -> Result<()> {
        crate::redis::repository::session::delete(ctx, id).await
    }
}
