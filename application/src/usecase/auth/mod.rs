use crate::interface::gateway::mail;
use crate::interface::gateway::mail::{MailGateway, UseMailGateway};
use crate::interface::repository::account::UseAccountRepository;
use crate::interface::repository::authentication::{
    AuthenticationRepository, PasswordResetCode, UpdatePassword, UseAuthenticationRepository,
};
use crate::interface::repository::session::UseSessionRepository;
use crate::interface::repository::Transaction;
use crate::interface::{Component, UseConfig, UseContext};
use crate::internal;
use crate::usecase::UseCase;
use anyhow::Result;
use anyhow::{bail, Context};
use async_trait::async_trait;
use helper::auth::stretch_password;
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
pub struct SignUpUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait SignUpUseCaseDeps<C: Component> = Component + UseContext<Context = C>
    + UseConfig
    + UseAuthenticationRepository<C>
    + UseSessionRepository<C>
    + UseMailGateway<C>
    ;
}
#[async_trait]
impl<C, Deps> UseCase<SignUpInput, SignUpOutput> for SignUpUseCase<C, Deps>
where
    C: Component,
    Deps: SignUpUseCaseDeps<C>,
{
    async fn handle(&self, input: SignUpInput) -> Result<SignUpOutput> {
        let ctx = self.deps.context().await?;
        let cfg = self.deps.config();
        let auth = self
            .deps
            .authentication_repository()
            .get_by_mail(ctx.clone(), input.mail.clone())
            .await?;
        if auth.is_some() {
            bail!(forbidden!("already exists. {}", input.mail.clone()))
        }
        let session = internal::session::new_provisional_account_session(
            &cfg,
            self.deps.session_repository(),
            ctx.clone(),
            input.mail.to_string(),
            input.password.to_string(),
        )
        .await?;

        let site_url = input.site_url;
        let signup_finish_url = format!("https://{}/signup/finish?code={}", site_url, session.code);

        let send_input = mail::SendEmailInput::new(
            format!("noreply@{}", cfg.system.mail_domain),
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
pub struct SignUpFinishUseCase<C, Deps> {
    deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait SignUpFinishUseCaseDeps<C: Transaction> = Component + UseContext<Context = C>
    + UseAccountRepository<C>
    + UseAuthenticationRepository<C>
    + UseSessionRepository<C>
    ;
}
#[async_trait]
impl<C, Deps> UseCase<SignUpFinishInput, SignUpFinishOutput> for SignUpFinishUseCase<C, Deps>
where
    C: Transaction,
    Deps: SignUpFinishUseCaseDeps<C>,
{
    async fn handle(&self, input: SignUpFinishInput) -> Result<SignUpFinishOutput> {
        let tx = self.deps.context().await?;
        let session_repo = self.deps.session_repository();
        let tx = tx.begin().await?;
        let session = internal::session::get_provisional_account_session(
            session_repo.clone(),
            tx.clone(),
            input.code.clone(),
        )
        .await?;
        let session: Session = match session {
            Some(session) if session.code == input.code => {
                // Save DB
                let account = internal::auth::signup_account(
                    self.deps.account_repository(),
                    self.deps.authentication_repository(),
                    tx.clone(),
                    session.authentication,
                )
                .await?;
                internal::session::new_session(session_repo, tx.clone(), account).await?
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
    + UseConfig
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
        let ctx = self.deps.context().await?;
        let account = internal::auth::signin_account(
            &self.deps.config(),
            self.deps.account_repository(),
            self.deps.authentication_repository(),
            ctx.clone(),
            input.mail,
            input.password,
        )
        .await?;
        let session =
            internal::session::new_session(self.deps.session_repository(), ctx.clone(), account)
                .await?;
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
        internal::session::invalidate_session(
            self.deps.session_repository(),
            ctx,
            input.session_id,
        )
        .await?;
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
    + UseConfig
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
        let auth_repo = self.deps.authentication_repository();
        let mail_gateway = self.deps.mail_gateway();
        let auth = auth_repo
            .get_by_mail(ctx.clone(), input.mail.clone())
            .await?;
        let Some(auth) = auth else {
            return Ok(ForgetPasswordOutput)
        };
        let code = helper::uuid::new_v4().to_base62();
        let password_reset_code = PasswordResetCode {
            code: code.clone(),
            mail: auth.mail.clone(),
        };
        auth_repo
            .add_password_reset_code(ctx.clone(), password_reset_code)
            .await?;
        let site_url = input.site_url;
        let password_reset_url = format!("https://{site_url}/reset_password?code={code}");
        let cfg = self.deps.config();
        let send_input = mail::SendEmailInput::new(
            format!("noreply@{}", cfg.system.mail_domain),
            input.mail.to_string(),
            "password reset link".to_string(),
            format!("password reset here! {password_reset_url}"),
        );
        mail_gateway
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
    + UseConfig
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
        let cfg = self.deps.config();
        let tx = self.deps.context().await?;
        let auth_repo = self.deps.authentication_repository();
        let tx = tx.begin().await?;
        let Some(password_reset_code) = auth_repo
            .get_password_reset_code(tx.clone(), input.code.clone())
            .await? else {
            bail!(unexpected!("invalid password reset code. code not found."))
        };
        let Some(authentication) =  auth_repo
            .get_by_mail(tx.clone(), password_reset_code.mail.clone())
            .await? else {
            bail!(unexpected!(
                "invalid password reset code. account not found"
            ))
        };
        let updated = UpdatePassword {
            account_id: authentication.account_id,
            mail: password_reset_code.mail,
            password: stretch_password(
                &input.password,
                &authentication.salt,
                &cfg.auth.pepper,
                cfg.auth.stretch_count,
            )?,
        };
        auth_repo.update_password(tx.clone(), updated).await?;
        tx.commit().await?;
        Ok(ResetPasswordOutput)
    }
}

#[derive(new)]
pub struct GetAuthStatusInput {
    pub(crate) session: Option<Session>,
}

#[derive(new, Debug, PartialEq)]
pub struct GetAuthStatusOutput;

#[derive(new)]
pub struct SignUpInput {
    pub(crate) mail: String,
    pub(crate) password: String,
    pub(crate) site_url: String,
}
#[derive(new, Debug, PartialEq)]
pub struct SignUpOutput;

#[derive(new)]
pub struct SignUpFinishInput {
    pub(crate) code: String,
}

#[derive(new, Debug, PartialEq)]
pub struct SignUpFinishOutput {
    pub session_id: String,
}

#[derive(new)]
pub struct SignInInput {
    pub(crate) mail: String,
    pub(crate) password: String,
    pub(crate) remember_me: bool,
}

#[derive(new, Debug, PartialEq)]
pub struct SignInOutput {
    pub session_id: String,
    pub remember_me: bool,
}

#[derive(new)]
pub struct SignOutInput {
    pub(crate) session_id: String,
}

#[derive(new, Debug, PartialEq)]
pub struct SignOutOutput;

#[derive(new)]
pub struct ForgetPasswordInput {
    pub(crate) mail: String,
    pub(crate) site_url: String,
}

#[derive(new, Debug, PartialEq)]
pub struct ForgetPasswordOutput;

#[derive(new)]
pub struct ResetPasswordInput {
    pub(crate) code: String,
    pub(crate) password: String,
}

#[derive(new, Debug, PartialEq)]
pub struct ResetPasswordOutput;

#[cfg(test)]
mod tests {
    use crate::interface::config::{AuthBuilder, Config, SystemBuilder};
    use crate::interface::gateway::mail::{MockMailGateway, UseMailGateway};
    use crate::interface::repository::authentication::{
        MockAuthenticationRepository, UseAuthenticationRepository,
    };
    use crate::interface::repository::session::{MockSessionRepository, UseSessionRepository};
    use crate::interface::{UseConfig, UseContext};
    use crate::usecase::auth::{SignUpInput, SignUpOutput, SignUpUseCase};
    use crate::usecase::UseCase;
    use async_trait::async_trait;
    use helper::auth::stretch_password;

    use kernel::Result;
    use mockall::predicate;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct TestMods {
        mock_auth_repo: Arc<MockAuthenticationRepository>,
        mock_session_repo: Arc<MockSessionRepository>,
        mock_mail_gateway: Arc<MockMailGateway>,
    }
    #[async_trait]
    impl UseContext for TestMods {
        type Context = ();
        async fn context(&self) -> Result<Self::Context> {
            Ok(())
        }
    }
    impl UseConfig for TestMods {
        fn config(&self) -> Config {
            Config::new(
                AuthBuilder::default()
                    .pepper("dummy_pepper".to_string())
                    .stretch_count(10)
                    .build()
                    .unwrap(),
                SystemBuilder::default()
                    .mail_domain("example.com")
                    .build()
                    .unwrap(),
            )
        }
    }
    impl UseAuthenticationRepository<()> for TestMods {
        type AuthenticationRepository = Arc<MockAuthenticationRepository>;

        fn authentication_repository(&self) -> Self::AuthenticationRepository {
            self.mock_auth_repo.clone()
        }
    }
    impl UseSessionRepository<()> for TestMods {
        type SessionRepository = Arc<MockSessionRepository>;

        fn session_repository(&self) -> Self::SessionRepository {
            self.mock_session_repo.clone()
        }
    }
    impl UseMailGateway<()> for TestMods {
        type Gateway = Arc<MockMailGateway>;

        fn mail_gateway(&self) -> Self::Gateway {
            self.mock_mail_gateway.clone()
        }
    }

    #[tokio::test]
    async fn signup() {
        let mut mock_auth_repo = MockAuthenticationRepository::default();
        let mut mock_session_repo = MockSessionRepository::default();
        let mut mock_mail_gateway = MockMailGateway::default();
        let mail = "mail@example.com";
        let password = "password";
        let site_url = "example.com";
        mock_auth_repo
            .expect_get_by_mail()
            .with(predicate::eq(()), predicate::eq(mail.to_string()))
            .return_once(move |_, _| Ok(None));
        let code: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let cloned_code = code.clone();
        mock_session_repo
            .expect_set_provisional_session()
            .with(predicate::eq(()), predicate::always())
            .return_once(move |_, session| {
                assert_eq!(mail.to_string(), session.authentication.mail);
                assert_eq!(
                    stretch_password(password, &session.authentication.salt, "dummy_pepper", 10)
                        .unwrap(),
                    session.authentication.password_hash
                );
                *cloned_code.lock().unwrap() = session.code;
                Ok(())
            });
        let cloned_code = code.clone();
        mock_mail_gateway
            .expect_send_email()
            .with(predicate::eq(()), predicate::always())
            .return_once(move |_, input| {
                assert_eq!("noreply@example.com", input.from_address);
                assert_eq!(mail.to_string(), input.to_address);
                assert_eq!("signup link", input.subject);
                let code = cloned_code.lock().unwrap().to_string();
                let body = format!("signup here! https://example.com/signup/finish?code={code}");
                assert_eq!(body, input.body);
                Ok(())
            });
        let mods = TestMods {
            mock_auth_repo: Arc::new(mock_auth_repo),
            mock_session_repo: Arc::new(mock_session_repo),
            mock_mail_gateway: Arc::new(mock_mail_gateway),
        };
        let interactor = SignUpUseCase::<(), TestMods>::new(mods);
        let input = SignUpInput::new(mail.to_string(), password.to_string(), site_url.to_string());
        let output = interactor.handle(input).await.unwrap();
        assert_eq!(output, SignUpOutput);
    }
}
