mod internal;
use crate::interface::gateway::mail;
use crate::interface::gateway::mail::{MailGateway, UseMailGateway};
use crate::interface::repository::account::UseAccountRepository;
use crate::interface::repository::authentication::{
    AuthenticationRepository, PasswordResetCode, UpdatePassword, UseAuthenticationRepository,
};
use crate::interface::repository::session::UseSessionRepository;
use crate::interface::repository::Transaction;
use crate::interface::{Component, UseContext};
use crate::usecase::UseCase;
use anyhow::Result;
use anyhow::{bail, Context};
use async_trait::async_trait;
use helper::auth::stretch_password;
use helper::env::get_var;
use helper::uuid::ToBase62;
use kernel::entity::Session;
use kernel::{forbidden, unexpected};
use std::marker::PhantomData;
use trait_set::trait_set;

#[derive(Clone, new)]
pub struct GetAuthStatusUseCase<C, Deps> {
    _deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait GetAuthStatusUseCaseDeps<C: Component> = Component + UseContext<Context = C>;
}

#[async_trait]
impl<C, Deps> UseCase<GetAuthStatusInput, GetAuthStatusOutput> for GetAuthStatusUseCase<C, Deps>
where
    C: Component,
    Deps: GetAuthStatusUseCaseDeps<C>,
{
    async fn handle(&self, input: GetAuthStatusInput) -> Result<GetAuthStatusOutput> {
        match input.session {
            Some(_) => Ok(GetAuthStatusOutput),
            None => bail!(forbidden!("invalid session")),
        }
    }
}

#[derive(Clone, new)]
pub struct SignupUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait SignupUseCaseDeps<C: Component> = Component + UseContext<Context = C>
    + UseAuthenticationRepository<C>
    + UseSessionRepository<C>
    + UseMailGateway<C>
    ;
}
#[async_trait]
impl<C, Deps> UseCase<SignUpInput, SignUpOutput> for SignupUseCase<C, Deps>
where
    C: Component,
    Deps: SignupUseCaseDeps<C>,
{
    async fn handle(&self, input: SignUpInput) -> Result<SignUpOutput> {
        let ctx = self.deps.context().await?;
        let auth = self
            .deps
            .authentication_repository()
            .get_by_mail(ctx.clone(), input.mail.clone())
            .await?;
        if auth.is_some() {
            return Ok(SignUpOutput);
        }
        let session = internal::new_preregister_session(
            ctx.clone(),
            self.deps.session_repository(),
            input.mail.to_string(),
            input.password.to_string(),
        )
        .await?;

        let site_url = input.site_url;
        let signup_finish_url = format!("https://{}/signup/finish?code={}", site_url, session.code);
        let send_input = mail::SendEmailInput::new(
            format!("noreply@{}", get_var::<String>("MAIL_DOMAIN")?),
            input.mail.to_string(),
            "signup link".to_string(),
            format!("signup here! {signup_finish_url}"),
        );
        self.deps
            .mail_gateway()
            .send_email(ctx.clone(), send_input)
            .await?;
        Ok(SignUpOutput)
    }
}

#[derive(Clone, new)]
pub struct SignupFinishUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait SignupFinishUseCaseDeps<C: Transaction> = Component + UseContext<Context = C>
    + UseAccountRepository<C>
    + UseAuthenticationRepository<C>
    + UseSessionRepository<C>
    ;
}
#[async_trait]
impl<C, Deps> UseCase<SignUpFinishInput, SignUpFinishOutput> for SignupFinishUseCase<C, Deps>
where
    C: Transaction,
    Deps: SignupFinishUseCaseDeps<C>,
{
    async fn handle(&self, input: SignUpFinishInput) -> Result<SignUpFinishOutput> {
        let tx = self.deps.context().await?;
        let tx = tx.begin().await?;
        let session = internal::get_preregister_session(
            tx.clone(),
            self.deps.session_repository(),
            input.code.clone(),
        )
        .await?;
        let session: Session = match session {
            Some(session) if session.code == input.code => {
                // Save DB
                let account = internal::signup_account(
                    tx.clone(),
                    self.deps.account_repository(),
                    self.deps.authentication_repository(),
                    session.mail,
                    session.password,
                )
                .await?;
                internal::new_session(tx.clone(), self.deps.session_repository(), account).await?
            }
            Some(s) => bail!(unexpected!("invalid session state: {:?}", s)),
            None => bail!(unexpected!("session not found.")),
        };
        tx.commit().await?;
        Ok(SignUpFinishOutput {
            session_id: session.id,
        })
    }
}

#[derive(Clone, new)]
pub struct SignInUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait SignInUseCaseDeps<C: Component> = Component + UseContext<Context = C>
    + UseAuthenticationRepository<C>
    + UseAccountRepository<C>
    + UseSessionRepository<C>
    ;
}
#[async_trait]
impl<C, Deps> UseCase<SignInInput, SignInOutput> for SignInUseCase<C, Deps>
where
    C: Component,
    Deps: SignInUseCaseDeps<C>,
{
    async fn handle(&self, input: SignInInput) -> Result<SignInOutput> {
        let db = self.deps.context().await?;
        let account = internal::signin_account(
            db.clone(),
            self.deps.account_repository(),
            self.deps.authentication_repository(),
            input.mail,
            input.password,
        )
        .await?;
        let session =
            internal::new_session(db.clone(), self.deps.session_repository(), account).await?;
        Ok(SignInOutput {
            session_id: session.id,
            remember_me: input.remember_me,
        })
    }
}

#[derive(Clone, new)]
pub struct SignOutUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait SignOutUseCaseDeps<C: Component> = Component + UseContext<Context = C>
    + UseSessionRepository<C>
    ;
}
#[async_trait]
impl<C, Deps> UseCase<SignOutInput, SignOutOutput> for SignOutUseCase<C, Deps>
where
    C: Component,
    Deps: SignOutUseCaseDeps<C>,
{
    async fn handle(&self, input: SignOutInput) -> Result<SignOutOutput> {
        let ctx = self.deps.context().await?;
        internal::invalidate_session(ctx, self.deps.session_repository(), input.session_id).await?;
        Ok(SignOutOutput)
    }
}

#[derive(Clone, new)]
pub struct ForgetPasswordUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait ForgetPasswordUseCaseDeps<C: Component> = Component + UseContext<Context = C>
    + UseAuthenticationRepository<C>
    + UseMailGateway<C>
    ;
}
#[async_trait]
impl<C, Deps> UseCase<ForgetPasswordInput, ForgetPasswordOutput> for ForgetPasswordUseCase<C, Deps>
where
    C: Component,
    Deps: ForgetPasswordUseCaseDeps<C>,
{
    async fn handle(&self, input: ForgetPasswordInput) -> Result<ForgetPasswordOutput> {
        let ctx = self.deps.context().await?;
        let auth = self
            .deps
            .authentication_repository()
            .get_by_mail(ctx.clone(), input.mail.clone())
            .await?;
        let auth = match auth {
            Some(auth) => auth,
            None => return Ok(ForgetPasswordOutput),
        };
        let code = helper::uuid::new_v4().to_base62();
        let password_reset_code = PasswordResetCode {
            code: code.clone(),
            mail: auth.mail.clone(),
        };
        self.deps
            .authentication_repository()
            .add_password_reset_code(ctx.clone(), password_reset_code)
            .await?;
        let site_url = input.site_url;
        let password_reset_url = format!("https://{site_url}/reset_password?code={code}");
        let send_input = mail::SendEmailInput::new(
            format!("noreply@{}", get_var::<String>("MAIL_DOMAIN")?),
            input.mail.to_string(),
            "password reset link".to_string(),
            format!("password reset here! {password_reset_url}"),
        );
        self.deps
            .mail_gateway()
            .send_email(ctx, send_input)
            .await
            .with_context(|| unexpected!("failed to send mail"))?;
        Ok(ForgetPasswordOutput)
    }
}

#[derive(Clone, new)]
pub struct ResetPasswordUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait ResetPasswordUseCaseDeps<C: Transaction> = Component + UseContext<Context = C>
    + UseAuthenticationRepository<C>
    + UseMailGateway<C>
    ;
}
#[async_trait]
impl<C, Deps> UseCase<ResetPasswordInput, ResetPasswordOutput> for ResetPasswordUseCase<C, Deps>
where
    C: Transaction,
    Deps: ResetPasswordUseCaseDeps<C>,
{
    async fn handle(&self, input: ResetPasswordInput) -> Result<ResetPasswordOutput> {
        let tx = self.deps.context().await?;
        let tx = tx.begin().await?;
        let password_reset_code = match self
            .deps
            .authentication_repository()
            .get_password_reset_code(tx.clone(), input.code.clone())
            .await?
        {
            Some(c) => c,
            None => bail!(unexpected!("invalid password reset code. code not found.")),
        };
        let authentication = match self
            .deps
            .authentication_repository()
            .get_by_mail(tx.clone(), password_reset_code.mail.clone())
            .await?
        {
            Some(auth) => auth,
            None => bail!(unexpected!(
                "invalid password reset code. account not found"
            )),
        };
        let updated = UpdatePassword {
            account_id: authentication.account_id,
            mail: password_reset_code.mail,
            password: stretch_password(&input.password, &authentication.salt)?,
        };
        self.deps
            .authentication_repository()
            .update_password(tx.clone(), updated)
            .await?;
        tx.commit().await?;
        Ok(ResetPasswordOutput)
    }
}

#[derive(new)]
pub struct GetAuthStatusInput {
    pub(crate) session: Option<Session>,
}

#[derive(new)]
pub struct GetAuthStatusOutput;

#[derive(new)]
pub struct SignUpInput {
    pub(crate) mail: String,
    pub(crate) password: String,
    pub(crate) site_url: String,
}
#[derive(new)]
pub struct SignUpOutput;

#[derive(new)]
pub struct SignUpFinishInput {
    pub(crate) code: String,
}

#[derive(new)]
pub struct SignUpFinishOutput {
    pub session_id: String,
}

#[derive(new)]
pub struct SignInInput {
    pub(crate) mail: String,
    pub(crate) password: String,
    pub(crate) remember_me: bool,
}

#[derive(new)]
pub struct SignInOutput {
    pub session_id: String,
    pub remember_me: bool,
}

#[derive(new)]
pub struct SignOutInput {
    pub(crate) session_id: String,
}

#[derive(new)]
pub struct SignOutOutput;

#[derive(new)]
pub struct ForgetPasswordInput {
    pub(crate) mail: String,
    pub(crate) site_url: String,
}

#[derive(new)]
pub struct ForgetPasswordOutput;

#[derive(new)]
pub struct ResetPasswordInput {
    pub(crate) code: String,
    pub(crate) password: String,
}

#[derive(new)]
pub struct ResetPasswordOutput;
