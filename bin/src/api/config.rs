use driver::http::server::api::config::Config as APIConfig;
use driver::mysql::config::Config as MySQLConfig;
use driver::redis::config::Config as RedisConfig;
use helper::env::get_var;
use kernel::Result;
use std::time::Duration;

#[derive(Clone)]
pub(crate) struct Config {
    pub(crate) api_config: APIConfig,
    pub(crate) mysql_config: MySQLConfig,
    pub(crate) redis_config: RedisConfig,
}

impl Config {
    pub fn new() -> Result<Config> {
        let config = Config {
            api_config: APIConfig::new(
                format!("0.0.0.0:{}", get_var::<u16>("LISTEN_PORT")?).parse()?,
            ),
            mysql_config: MySQLConfig::new(
                get_var("DATABASE_URL")?,
                get_var("DATABASE_MIN_CONNECTIONS")?,
                get_var("DATABASE_MAX_CONNECTIONS")?,
                Duration::from_secs(get_var("DATABASE_CONNECT_TIMEOUT")?),
                Duration::from_secs(get_var("DATABASE_IDLE_TIMEOUT")?),
                Duration::from_secs(get_var("DATABASE_MAX_LIFETIME")?),
            ),
            redis_config: RedisConfig::new(
                get_var("REDIS_PRIMARY_URL")?,
                get_var("REDIS_READER_URL")?,
                Some(get_var("REDIS_MIN_IDLE")?),
                get_var("REDIS_MAX_SIZE")?,
            ),
        };
        Ok(config)
    }
}
