use crate::interface::repository::account::{AccountRepository, NewAccount};
use crate::interface::repository::authentication::AuthenticationRepository;
use crate::interface::repository::session::SessionRepository;
use crate::interface::{repository, Component};
use anyhow::bail;
use helper::auth::stretch_password;
use helper::uuid;
use helper::uuid::ToBase62;
use kernel::entity::{Account, ProvisionalSession, Session};
use kernel::error::Codes;
use kernel::Result;
use kernel::{forbidden, unexpected};

pub async fn signup_account<C: Component>(
    db: C,
    account_repo: impl AccountRepository<C>,
    auth_repo: impl AuthenticationRepository<C>,
    mail: String,
    password: String,
) -> Result<Account> {
    let id = uuid::new_v4().to_base62();
    let salt = uuid::new_v4().to_base62();
    let new_account = NewAccount {
        id: id.to_string(),
        name: id.to_string(),
        display_name: id.to_string(),
    };
    let account = account_repo.create(db.clone(), new_account.clone()).await?;
    let new_auth = repository::authentication::NewAuthentication {
        account_id: id.to_string(),
        mail,
        salt: salt.to_string(),
        password: stretch_password(&password, &salt)?,
    };
    auth_repo.create(db.clone(), new_auth).await?;
    Ok(account)
}

pub async fn signin_account<C: Component>(
    db: C,
    repo: impl AccountRepository<C>,
    auth_repo: impl AuthenticationRepository<C>,
    mail: String,
    password: String,
) -> Result<Account> {
    let Some(authentication) = auth_repo.get_by_mail(db.clone(), mail.to_string()).await? else {
        let _ = stretch_password(&password, "dummy")?; // dummy stretching.
        bail!(forbidden!("inputs: {}/{}", mail, password)
                .with_codes(Codes::InvalidEmailOrPassword))
    };

    let hash = stretch_password(&password, &authentication.salt)?;
    if hash != authentication.password {
        bail!(forbidden!("inputs: {}/{}", mail, password).with_codes(Codes::InvalidEmailOrPassword))
    }
    let Some(account) = repo.get(db.clone(), authentication.account_id).await? else {
        bail!(unexpected!("Account NotFound"))
    };
    Ok(account)
}

pub async fn new_preregister_session<C>(
    ctx: C,
    repo: impl SessionRepository<C>,
    mail: String,
    password: String,
) -> Result<ProvisionalSession> {
    let code = helper::uuid::new_v4().to_base62();
    let session = ProvisionalSession {
        code,
        mail,
        password,
    };
    repo.set_provisional_session(ctx, session.clone()).await?;
    Ok(session)
}

pub async fn get_preregister_session<C>(
    ctx: C,
    repo: impl SessionRepository<C>,
    id: String,
) -> Result<Option<ProvisionalSession>> {
    repo.get_provisional_session(ctx, id).await
}

pub async fn new_session<C>(
    ctx: C,
    repo: impl SessionRepository<C>,
    account: Account,
) -> Result<Session> {
    let id = helper::uuid::new_v4().to_base62();
    let time = helper::time::current_time();
    let session = Session {
        id: id.to_string(),
        account,
        create_time: time,
    };
    repo.set(ctx, session.clone()).await?;
    Ok(session)
}

pub async fn invalidate_session<C>(
    ctx: C,
    repo: impl SessionRepository<C>,
    id: String,
) -> Result<()> {
    repo.delete(ctx, id).await
}
