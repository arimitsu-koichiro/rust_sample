use crate::mysql::MySQLContext;
use crate::redis::RedisContext;
use application::interface::repository::authentication::{
    AuthenticationRepository, NewAuthentication, PasswordResetCode, UpdatePassword,
};
use async_trait::async_trait;
use derive_new::new;
use kernel::entity;
use kernel::Result;

#[derive(Clone, Debug, new)]
pub struct AuthenticationRepositoryImpl;

#[async_trait]
impl<Context> AuthenticationRepository<Context> for AuthenticationRepositoryImpl
where
    Context: MySQLContext + RedisContext,
{
    async fn get_by_mail(
        &self,
        ctx: Context,
        mail: String,
    ) -> Result<Option<entity::Authentication>> {
        crate::mysql::repository::authentication::get_by_mail(ctx, mail).await
    }
    async fn create(&self, ctx: Context, new_authentication: NewAuthentication) -> Result<()> {
        crate::mysql::repository::authentication::create(ctx, new_authentication).await
    }
    async fn update_password(&self, ctx: Context, updated: UpdatePassword) -> Result<()> {
        crate::mysql::repository::authentication::update_password(ctx, updated).await
    }
    async fn add_password_reset_code(
        &self,
        ctx: Context,
        password_reset_code: PasswordResetCode,
    ) -> Result<()> {
        crate::redis::repository::authentication::add_password_reset_code(ctx, password_reset_code)
            .await
    }
    async fn get_password_reset_code(
        &self,
        ctx: Context,
        code: String,
    ) -> Result<Option<PasswordResetCode>> {
        crate::redis::repository::authentication::get_password_reset_code(ctx, code).await
    }
}
