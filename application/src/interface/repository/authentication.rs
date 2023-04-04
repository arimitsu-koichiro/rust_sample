use crate::interface::Component;
use async_trait::async_trait;
use blanket::blanket;
use kernel::entity::Authentication;
use kernel::Result;
#[cfg(test)]
use mockall::mock;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[async_trait]
#[blanket(derive(Arc))]
pub trait AuthenticationRepository<Context>: Component {
    async fn get_by_mail(&self, ctx: Context, mail: String) -> Result<Option<Authentication>>;
    async fn create(&self, ctx: Context, authentication: Authentication) -> Result<()>;
    async fn update_password(&self, ctx: Context, updated: UpdatePassword) -> Result<()>;
    async fn add_password_reset_code(
        &self,
        ctx: Context,
        password_reset_code: PasswordResetCode,
    ) -> Result<()>;
    async fn get_password_reset_code(
        &self,
        ctx: Context,
        code: String,
    ) -> Result<Option<PasswordResetCode>>;
}

pub trait UseAuthenticationRepository<Context> {
    type AuthenticationRepository: AuthenticationRepository<Context>;
    fn authentication_repository(&self) -> Self::AuthenticationRepository;
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, new)]
pub struct PasswordResetCode {
    pub code: String,
    #[validate(email)]
    pub mail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, new)]
pub struct UpdatePassword {
    pub account_id: String,
    #[validate(email)]
    pub mail: String,
    pub password: String,
}

#[cfg(test)]
mock! {
    pub AuthenticationRepository{}
    impl Clone for AuthenticationRepository {
        fn clone(&self) -> Self;
    }
    #[async_trait]
    impl AuthenticationRepository<()> for AuthenticationRepository {
        async fn get_by_mail(&self, ctx: (), mail: String) -> Result<Option<Authentication>>;
        async fn create(&self, ctx: (), authentication: Authentication) -> Result<()>;
        async fn update_password(&self, ctx: (), updated: UpdatePassword) -> Result<()>;
        async fn add_password_reset_code(
            &self,
            ctx: (),
            password_reset_code: PasswordResetCode,
        ) -> Result<()>;
        async fn get_password_reset_code(
            &self,
            ctx: (),
            code: String,
        ) -> Result<Option<PasswordResetCode>>;
    }
}
