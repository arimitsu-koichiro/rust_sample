use std::fmt::Debug;

use crate::mysql::MySQLContext;
use application::interface::repository::account::{AccountRepository, NewAccount};
use async_trait::async_trait;
use kernel::entity;
use kernel::Result;

#[derive(Clone, Debug)]
pub struct AccountRepositoryImpl;

#[async_trait]
impl<Context> AccountRepository<Context> for AccountRepositoryImpl
where
    Context: MySQLContext,
{
    async fn get(&self, ctx: Context, id: String) -> Result<Option<entity::Account>> {
        crate::mysql::repository::account::get(ctx, id).await
    }

    async fn create(&self, ctx: Context, new_account: NewAccount) -> Result<entity::Account> {
        crate::mysql::repository::account::create(ctx, new_account).await
    }
}
