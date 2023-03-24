use crate::interface::Component;
use async_trait::async_trait;
use blanket::blanket;
use kernel::entity::{ProvisionalSession, Session};
use kernel::Result;

#[async_trait]
#[blanket(derive(Arc))]
pub trait SessionRepository<Context>: Component {
    async fn set_provisional_session(
        &self,
        ctx: Context,
        session: ProvisionalSession,
    ) -> Result<()>;
    async fn get_provisional_session(
        &self,
        ctx: Context,
        id: String,
    ) -> Result<Option<ProvisionalSession>>;
    async fn set(&self, ctx: Context, session: Session) -> Result<()>;
    async fn get(&self, ctx: Context, id: String) -> Result<Option<Session>>;
    async fn delete(&self, ctx: Context, id: String) -> Result<()>;
}

pub trait UseSessionRepository<Context> {
    type SessionRepository: SessionRepository<Context>;
    fn session_repository(&self) -> Self::SessionRepository;
}
