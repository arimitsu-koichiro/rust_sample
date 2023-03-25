use application::interface::gateway::pubsub::UsePubSubGateway;
use application::interface::UseContext;
use application::usecase::channel::{SubscribeInput, SubscribeOutput, SubscribeUseCase};
use application::usecase::UseUseCase;
use async_trait::async_trait;
use derive_new::new;
use driver::adapter::gateway::pubsub::PubSubGatewayImpl;
use driver::aws::ssm::load_from_ssm;
use driver::cli::presenter::logging::LoggingPresenter;
use driver::redis::config::RedisConfig;
use driver::redis::{
    PooledRedisConnection, Redis, RedisConnection, RedisConnectionManager, RedisPrimaryContext,
    RedisReaderContext,
};
use driver::UsePresenter;
use helper::env::get_var;
use kernel::Result;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    if let Ok(ssm_envs_path) = get_var("SSM_ENVS_PATH") {
        load_from_ssm(ssm_envs_path).await?;
    }
    log::init();
    let config = Config::new();
    let redis = Redis::new(config.redis_config).await?;
    driver::cli::listen_pubsub(Modules::new(redis)).await?;
    Ok(())
}

#[derive(Clone, new)]
struct Modules {
    redis: Redis,
}

impl UsePresenter for Modules {
    type Presenter = LoggingPresenter;

    fn presenter(&self) -> Self::Presenter {
        LoggingPresenter::default()
    }
}

#[async_trait]
impl UseContext for Modules {
    type Context = Context;

    async fn context(&self) -> Result<Self::Context> {
        Ok(Context {
            redis: self.redis.clone(),
        })
    }
}

impl UseUseCase<SubscribeInput, SubscribeOutput> for Modules {
    type UseCase = SubscribeUseCase<Context, Modules>;

    fn usecase(&self) -> Self::UseCase {
        SubscribeUseCase::new(self.clone())
    }
}

impl UsePubSubGateway<Context> for Modules {
    type Gateway = PubSubGatewayImpl;

    fn pubsub_gateway(&self) -> Self::Gateway {
        PubSubGatewayImpl
    }
}

#[derive(Clone)]
pub struct Context {
    redis: Redis,
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

mod log {
    use helper::env::get_var_or;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    pub fn init() {
        tracing_subscriber::registry()
            .with(tracing_subscriber::filter::LevelFilter::INFO)
            .with(
                tracing_subscriber::fmt::Layer::default()
                    .with_line_number(true)
                    .with_file(true)
                    .with_ansi(get_var_or("LOG_COLOR", true)),
            )
            .init();
    }
}

#[derive(Clone)]
struct Config {
    pub redis_config: RedisConfig,
}

impl Config {
    pub fn new() -> Config {
        compose_config().unwrap()
    }
}
fn compose_config() -> Result<Config> {
    let config = Config {
        redis_config: RedisConfig::new(
            get_var("REDIS_PRIMARY_URL")?,
            get_var("REDIS_READER_URL")?,
            Some(get_var("REDIS_MIN_IDLE")?),
            get_var("REDIS_MAX_SIZE")?,
        ),
    };
    Ok(config)
}
