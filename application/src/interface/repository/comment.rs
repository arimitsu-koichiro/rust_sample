use crate::interface::Component;
use async_trait::async_trait;
use blanket::blanket;
use kernel::entity::Comment;
use kernel::Result;

#[async_trait]
#[blanket(derive(Arc))]
pub trait CommentRepository<Context>: Component {
    async fn get(&self, ctx: Context, id: String) -> Result<Option<Comment>>;
    async fn put(&self, ctx: Context, id: String, body: String) -> Result<Option<Comment>>;
}

pub trait UseCommentRepository<Context> {
    type CommentRepository: CommentRepository<Context>;
    fn comment_repository(&self) -> Self::CommentRepository;
}
