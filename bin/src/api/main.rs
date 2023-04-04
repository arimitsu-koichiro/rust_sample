use crate::modules::Modules;
use application::interface::config::{AuthBuilder, ConfigBuilder, SystemBuilder};
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
    let cfg = ConfigBuilder::default()
        .auth(
            AuthBuilder::default()
                .pepper(get_var::<String>("AUTH_PEPPER").unwrap())
                .stretch_count(get_var::<i64>("AUTH_STRETCH_COUNT").unwrap())
                .build()
                .unwrap(),
        )
        .system(
            SystemBuilder::default()
                .mail_domain(get_var::<String>("MAIL_DOMAIN").unwrap())
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();
    driver::http::server::api::start(config.api_config, Modules::new(cfg, db, redis)).await?;
    Ok(())
}

mod log {
    use helper::env::{get_var_or_else, var_is};
    use tracing_subscriber::filter::LevelFilter;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{filter, registry};

    pub fn init() {
        let log_format = tracing_subscriber::fmt::Layer::default()
            .with_line_number(true)
            .with_file(true)
            .with_ansi(var_is("LOG_COLOR", true));
        let filter = get_var_or_else::<filter::Targets>("RUST_LOG", || {
            filter::Targets::new().with_default(LevelFilter::INFO)
        });
        if var_is::<String>("LOG_FORMAT", "json".to_string()) {
            registry().with(filter).with(log_format.json()).init();
        } else {
            registry().with(filter).with(log_format).init();
        }
    }
}
