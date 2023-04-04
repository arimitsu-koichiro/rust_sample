pub mod session {
    use crate::interface::config::Config;
    use crate::interface::repository::session::SessionRepository;
    use helper::auth::stretch_password;
    use helper::time::current_time;
    use helper::uuid;
    use helper::uuid::ToBase62;
    use helper::validation::Validation;
    use kernel::entity::{Account, ProvisionalAuthentication, ProvisionalSession, Session};
    use kernel::Result;

    pub async fn new_session<C>(
        repo: impl SessionRepository<C>,
        ctx: C,
        account: Account,
    ) -> Result<Session> {
        let id = uuid::new_v4().to_base62();
        let session = Session::new(id.to_string(), account, current_time()).validate()?;
        repo.set(ctx, session.clone()).await?;
        Ok(session)
    }
    pub async fn get_session<C>(
        repo: impl SessionRepository<C>,
        ctx: C,
        id: String,
    ) -> Result<Option<Session>> {
        repo.get(ctx, id).await
    }
    pub async fn invalidate_session<C>(
        repo: impl SessionRepository<C>,
        ctx: C,
        id: String,
    ) -> Result<()> {
        repo.delete(ctx, id).await
    }
    pub async fn new_provisional_account_session<C>(
        cfg: &Config,
        repo: impl SessionRepository<C>,
        ctx: C,
        mail: String,
        password: String,
    ) -> Result<ProvisionalSession> {
        let code = uuid::new_v4().to_base62();
        let salt = uuid::new_v4().to_base62();
        let password_hash =
            stretch_password(&password, &salt, &cfg.auth.pepper, cfg.auth.stretch_count)?;
        let auth = ProvisionalAuthentication::new(mail, salt, password_hash).validate()?;
        let session = ProvisionalSession::new(code, auth).validate()?;
        repo.set_provisional_session(ctx, session.clone()).await?;
        Ok(session)
    }
    pub async fn get_provisional_account_session<C>(
        repo: impl SessionRepository<C>,
        ctx: C,
        id: String,
    ) -> Result<Option<ProvisionalSession>> {
        repo.get_provisional_session(ctx, id).await
    }
}

pub mod auth {
    use crate::interface::config::Config;
    use crate::interface::repository::account::AccountRepository;
    use crate::interface::repository::authentication::AuthenticationRepository;
    use crate::interface::Component;
    use anyhow::bail;
    use helper::auth::stretch_password;
    use helper::time::current_time;
    use helper::uuid;
    use helper::uuid::ToBase62;
    use helper::validation::Validation;
    use kernel::entity::{Account, Authentication, ProvisionalAuthentication};
    use kernel::error::Codes;
    use kernel::Result;
    use kernel::{forbidden, unexpected};

    pub async fn signup_account<C: Component>(
        account_repo: impl AccountRepository<C>,
        auth_repo: impl AuthenticationRepository<C>,
        ctx: C,
        ProvisionalAuthentication {
            mail,
            salt,
            password_hash,
        }: ProvisionalAuthentication,
    ) -> Result<Account> {
        let id = uuid::new_v4().to_base62();
        let new_account = Account::new(
            id.to_string(),
            id.to_string(),
            id.to_string(),
            current_time(),
        )
        .validate()?;
        let account = account_repo.create(ctx.clone(), new_account).await?;
        let new_auth = Authentication::new(id.clone(), mail, salt, password_hash).validate()?;
        auth_repo.create(ctx.clone(), new_auth).await?;
        Ok(account)
    }

    pub async fn signin_account<C: Component>(
        cfg: &Config,
        account_repo: impl AccountRepository<C>,
        auth_repo: impl AuthenticationRepository<C>,
        ctx: C,
        mail: String,
        password: String,
    ) -> Result<Account> {
        let Some(authentication) = auth_repo.get_by_mail(ctx.clone(), mail.to_string()).await? else {
            let _ = stretch_password(&password, "dummy", &cfg.auth.pepper, cfg.auth.stretch_count)?; // dummy stretching.
            bail!(forbidden!("inputs: {}/{}", mail, password)
                .with_codes(Codes::InvalidEmailOrPassword))
        };

        let hash = stretch_password(
            &password,
            &authentication.salt,
            &cfg.auth.pepper,
            cfg.auth.stretch_count,
        )?;
        if hash != authentication.password_hash {
            bail!(forbidden!("inputs: {}/{}", mail, password)
                .with_codes(Codes::InvalidEmailOrPassword))
        }
        let Some(account) = account_repo.get(ctx.clone(), authentication.account_id).await? else {
            bail!(unexpected!("Account NotFound"))
        };
        Ok(account)
    }
}
