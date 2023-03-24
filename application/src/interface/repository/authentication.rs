use crate::interface::Component;
use async_trait::async_trait;
use blanket::blanket;
use kernel::entity::Authentication;
use kernel::Result;
use serde::{Deserialize, Serialize};

#[async_trait]
#[blanket(derive(Arc))]
pub trait AuthenticationRepository<Context>: Component {
    async fn get_by_mail(&self, ctx: Context, mail: String) -> Result<Option<Authentication>>;
    async fn create(&self, ctx: Context, authentication: NewAuthentication) -> Result<()>;
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

#[derive(Debug, Clone, Serialize, Deserialize, new)]
pub struct PasswordResetCode {
    pub code: String,
    pub mail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, new)]
pub struct UpdatePassword {
    pub account_id: String,
    pub mail: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, new)]
pub struct NewAuthentication {
    pub account_id: String,
    pub mail: String,
    pub salt: String,
    pub password: String,
}
