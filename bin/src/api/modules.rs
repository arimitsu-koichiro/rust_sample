use application::interface::gateway::mail::UseMailGateway;
use application::interface::gateway::pubsub::UsePubSubGateway;
use application::interface::repository::account::UseAccountRepository;
use application::interface::repository::authentication::UseAuthenticationRepository;
use application::interface::repository::comment::UseCommentRepository;
use application::interface::repository::session::UseSessionRepository;
use application::interface::repository::Transaction;
use application::interface::UseContext;
use application::usecase::account::*;
use application::usecase::auth::*;
use application::usecase::channel::*;
use application::usecase::session::*;
use application::usecase::status::*;
use application::usecase::UseUseCase;
use async_trait::async_trait;
use driver::adapter::gateway::mail::{SesContext, SesMailGateway};
use driver::adapter::gateway::pubsub::PubSubGatewayImpl;
use driver::adapter::repository::account::AccountRepositoryImpl;
use driver::adapter::repository::authentication::AuthenticationRepositoryImpl;
use driver::adapter::repository::comment::CommentRepositoryImpl;
use driver::adapter::repository::session::SessionRepositoryImpl;
use driver::http::server::api::presenter::openapi::OpenAPIServerPresenter;
use driver::mysql::{MySQLContext, MySql, PoolConnection, DB};
use driver::redis::{
    PooledRedisConnection, Redis, RedisConnection, RedisConnectionManager, RedisPrimaryContext,
    RedisReaderContext,
};
use driver::UsePresenter;
use kernel::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub(crate) struct Modules {
    pub(crate) db: DB,
    pub(crate) redis: Redis,
}

impl Modules {
    #[allow(clippy::redundant_clone)]
    pub fn new(db: DB, redis: Redis) -> Modules {
        Modules { db, redis }
    }
}

#[derive(Clone)]
pub(crate) struct Context {
    db: DB,
    redis: Redis,
}

#[async_trait]
impl Transaction for Context {
    async fn begin(&self) -> Result<Context> {
        Ok(Context {
            db: self.db.begin().await?,
            redis: self.redis.clone(),
        })
    }

    async fn commit(&self) -> Result<Context> {
        Ok(Context {
            db: self.db.commit().await?,
            redis: self.redis.clone(),
        })
    }
}

#[async_trait]
impl MySQLContext for Context {
    async fn acquire(&self) -> Result<Arc<Mutex<PoolConnection<MySql>>>> {
        self.db.acquire().await
    }
}

#[async_trait]
impl RedisPrimaryContext for Context {
    async fn primary(&self) -> Result<PooledRedisConnection<RedisConnectionManager>> {
        self.redis.primary().await
    }
}

#[async_trait]
impl RedisReaderContext for Context {
    async fn reader(&self) -> Result<PooledRedisConnection<RedisConnectionManager>> {
        self.redis.reader().await
    }

    async fn subscribe_connection(&self) -> Result<RedisConnection> {
        self.redis.subscribe_connection().await
    }
}

#[async_trait]
impl SesContext for Context {}

#[async_trait]
impl UseContext for Modules {
    type Context = Context;

    async fn context(&self) -> Result<Self::Context> {
        Ok(Context {
            db: self.db.clone(),
            redis: self.redis.clone(),
        })
    }
}

impl UseAccountRepository<Context> for Modules {
    type AccountRepository = AccountRepositoryImpl;

    fn account_repository(&self) -> Self::AccountRepository {
        AccountRepositoryImpl
    }
}
impl UseSessionRepository<Context> for Modules {
    type SessionRepository = SessionRepositoryImpl;

    fn session_repository(&self) -> Self::SessionRepository {
        SessionRepositoryImpl
    }
}

impl UseAuthenticationRepository<Context> for Modules {
    type AuthenticationRepository = AuthenticationRepositoryImpl;

    fn authentication_repository(&self) -> Self::AuthenticationRepository {
        AuthenticationRepositoryImpl
    }
}

impl UseCommentRepository<Context> for Modules {
    type CommentRepository = CommentRepositoryImpl;

    fn comment_repository(&self) -> Self::CommentRepository {
        CommentRepositoryImpl
    }
}

impl UseMailGateway<Context> for Modules {
    type Gateway = SesMailGateway;

    fn mail_gateway(&self) -> Self::Gateway {
        SesMailGateway
    }
}

impl UsePubSubGateway<Context> for Modules {
    type Gateway = PubSubGatewayImpl;

    fn pubsub_gateway(&self) -> Self::Gateway {
        PubSubGatewayImpl
    }
}

impl UsePresenter for Modules {
    type Presenter = OpenAPIServerPresenter;

    fn presenter(&self) -> Self::Presenter {
        OpenAPIServerPresenter::default()
    }
}

impl UseUseCase<StatusInput, StatusOutput> for Modules {
    type UseCase = StatusUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        StatusUseCase::new(self.clone())
    }
}
impl UseUseCase<GetAccountInput, GetAccountOutput> for Modules {
    type UseCase = GetAccountUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        GetAccountUseCase::new(self.clone())
    }
}
impl UseUseCase<GetAuthStatusInput, GetAuthStatusOutput> for Modules {
    type UseCase = GetAuthStatusUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        GetAuthStatusUseCase::new(self.clone())
    }
}
impl UseUseCase<SignUpInput, SignUpOutput> for Modules {
    type UseCase = SignUpUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        SignUpUseCase::new(self.clone())
    }
}
impl UseUseCase<SignUpFinishInput, SignUpFinishOutput> for Modules {
    type UseCase = SignUpFinishUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        SignUpFinishUseCase::new(self.clone())
    }
}
impl UseUseCase<SignInInput, SignInOutput> for Modules {
    type UseCase = SignInUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        SignInUseCase::new(self.clone())
    }
}
impl UseUseCase<SignOutInput, SignOutOutput> for Modules {
    type UseCase = SignOutUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        SignOutUseCase::new(self.clone())
    }
}
impl UseUseCase<ForgetPasswordInput, ForgetPasswordOutput> for Modules {
    type UseCase = ForgetPasswordUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        ForgetPasswordUseCase::new(self.clone())
    }
}
impl UseUseCase<ResetPasswordInput, ResetPasswordOutput> for Modules {
    type UseCase = ResetPasswordUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        ResetPasswordUseCase::new(self.clone())
    }
}
impl UseUseCase<PublishInput, PublishOutput> for Modules {
    type UseCase = PublishUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        PublishUseCase::new(self.clone())
    }
}
impl UseUseCase<SubscribeInput, SubscribeOutput> for Modules {
    type UseCase = SubscribeUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        SubscribeUseCase::new(self.clone())
    }
}
impl UseUseCase<PubSubInput, PubSubOutput> for Modules {
    type UseCase = PubSubUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        PubSubUseCase::new(self.clone())
    }
}
impl UseUseCase<GetSessionInput, GetSessionOutput> for Modules {
    type UseCase = GetSessionUseCase<Context, Modules>;
    fn usecase(&self) -> Self::UseCase {
        GetSessionUseCase::new(self.clone())
    }
}
