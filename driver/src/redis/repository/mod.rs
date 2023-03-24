pub mod authentication {
    use crate::redis::{compose_key, RedisPrimaryContext, RedisReaderContext};
    use anyhow::Context as _;
    use application::interface::repository::authentication::PasswordResetCode;

    use helper::env::get_var;
    use kernel::{unexpected, Result};
    use redis::AsyncCommands;

    pub async fn add_password_reset_code(
        ctx: impl RedisPrimaryContext,
        password_reset_code: PasswordResetCode,
    ) -> Result<()> {
        let mut conn = ctx.primary().await?;

        conn.set_ex(
            compose_key("password_reset_code", &password_reset_code.code),
            helper::json::to_vec(&password_reset_code)?.as_slice(),
            get_var("PASSWORD_RESET_CODE_EXPIRE")?,
        )
        .await
        .with_context(|| unexpected!("add_password_reset_code set_ex error"))
    }
    pub async fn get_password_reset_code(
        ctx: impl RedisReaderContext,
        code: String,
    ) -> Result<Option<PasswordResetCode>> {
        let mut conn = ctx.reader().await?;
        match conn
            .get::<_, Vec<u8>>(compose_key("password_reset_code", &code))
            .await
            .ok()
        {
            Some(x) => Ok(Some(helper::json::from_vec::<PasswordResetCode>(&x)?)),
            None => Ok(None),
        }
    }
}
pub mod session {
    use crate::redis::{compose_key, RedisPrimaryContext, RedisReaderContext};
    use anyhow::Context as _;

    use kernel::entity::{ProvisionalSession, Session};

    use kernel::{unexpected, Result};
    use redis::AsyncCommands;

    pub async fn set_provisional_session(
        ctx: impl RedisPrimaryContext,
        session: ProvisionalSession,
    ) -> Result<()> {
        let mut conn = ctx.primary().await?;
        conn.set(
            compose_key("preregister", &session.code),
            helper::json::to_vec(&session)?.as_slice(),
        )
        .await
        .with_context(|| unexpected!("set_provisional_session error"))
    }
    pub async fn get_provisional_session(
        ctx: impl RedisReaderContext,
        id: String,
    ) -> Result<Option<ProvisionalSession>> {
        let mut conn = ctx.reader().await?;
        let x = conn
            .get::<_, Vec<u8>>(compose_key("preregister", &id))
            .await
            .with_context(|| unexpected!("get_provisional_session error"))?;
        Ok(Some(helper::json::from_vec::<ProvisionalSession>(&x)?))
    }
    pub async fn set(ctx: impl RedisPrimaryContext, session: Session) -> Result<()> {
        let mut conn = ctx.primary().await?;
        conn.set(
            compose_key("session", &session.id),
            helper::json::to_vec(&session)?.as_slice(),
        )
        .await
        .with_context(|| unexpected!("session set error"))
    }

    pub async fn get(ctx: impl RedisReaderContext, id: String) -> Result<Option<Session>> {
        let mut conn = ctx.reader().await?;
        let x = conn
            .get::<_, Vec<u8>>(compose_key("session", &id))
            .await
            .with_context(|| unexpected!("session get error"))?;
        Ok(Some(helper::json::from_vec::<Session>(&x)?))
    }
    pub async fn delete(ctx: impl RedisPrimaryContext, id: String) -> Result<()> {
        let mut conn = ctx.primary().await?;
        conn.del(compose_key("session", &id))
            .await
            .with_context(|| unexpected!("session delete error"))?;
        Ok(())
    }
}
