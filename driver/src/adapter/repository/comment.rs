use std::fmt::Debug;

use crate::mysql::MySQLContext;
use application::interface::repository::comment::CommentRepository;
use async_trait::async_trait;
use kernel::entity;
use kernel::Result;

#[derive(Clone, Debug)]
pub struct CommentRepositoryImpl;

#[async_trait]
impl<Context> CommentRepository<Context> for CommentRepositoryImpl
where
    Context: MySQLContext,
{
    async fn get(&self, ctx: Context, id: String) -> Result<Option<entity::Comment>> {
        crate::mysql::repository::comment::get(ctx, id).await
    }

    async fn put(&self, ctx: Context, id: String, body: String) -> Result<Option<entity::Comment>> {
        crate::mysql::repository::comment::put(ctx, id, body).await
    }
}
