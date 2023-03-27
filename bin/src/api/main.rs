use crate::modules::Modules;
use driver::aws::ssm::load_from_ssm;
use driver::mysql::DB;
use driver::redis::Redis;
use helper::env::get_var;
use kernel::Result;
mod config;
mod modules;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    if let Ok(ssm_envs_path) = get_var("SSM_ENVS_PATH") {
        load_from_ssm(ssm_envs_path).await?;
    }
    log::init();
    let config = config::Config::new()?;
    let db = DB::new(config.mysql_config).await?;
    let redis = Redis::new(config.redis_config).await?;
    driver::http::server::api::start(config.api_config, Modules::new(db, redis)).await?;
    Ok(())
}

mod log {
    use helper::env::{get_var_or, var_is};
    use tracing_subscriber::filter::LevelFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    pub fn init() {
        let log_format = tracing_subscriber::fmt::Layer::default()
            .with_line_number(true)
            .with_file(true)
            .with_ansi(var_is("LOG_COLOR", true));
        let level = get_var_or::<LevelFilter>("RUST_LOG", LevelFilter::INFO);
        if var_is::<String>("LOG_FORMAT", "json".to_string()) {
            tracing_subscriber::registry()
                .with(level)
                .with(log_format.json())
                .init();
        } else {
            tracing_subscriber::registry()
                .with(level)
                .with(log_format)
                .init();
        }
    }
}
