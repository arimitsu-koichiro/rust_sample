use crate::interface::Component;
use async_trait::async_trait;
use blanket::blanket;
use kernel::entity::{ProvisionalSession, Session};
use kernel::Result;
#[cfg(test)]
use mockall::mock;

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

#[cfg(test)]
mock! {
    pub SessionRepository{}
    impl Clone for SessionRepository {
        fn clone(&self) -> Self;
    }
    #[async_trait]
    impl SessionRepository<()> for SessionRepository {
    async fn set_provisional_session(
        &self,
        ctx: (),
        session: ProvisionalSession,
    ) -> Result<()>;
    async fn get_provisional_session(
        &self,
        ctx: (),
        id: String,
    ) -> Result<Option<ProvisionalSession>>;
    async fn set(&self, ctx: (), session: Session) -> Result<()>;
    async fn get(&self, ctx: (), id: String) -> Result<Option<Session>>;
    async fn delete(&self, ctx: (), id: String) -> Result<()>;
    }
}
