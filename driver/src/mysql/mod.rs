use crate::mysql::config::MySQLConfig;
use anyhow::{Context as _, Result};
use application::interface::repository::Transaction;
use application::interface::{Component, UseContext};
use async_trait::async_trait;
use kernel::unexpected;
use log;
pub use sqlx::mysql::MySqlPoolOptions;
pub use sqlx::pool::PoolConnection;
pub use sqlx::MySql;
pub use sqlx::Pool as MySqlPool;
use sqlx::TransactionManager;

use std::sync::Arc;
use tokio::sync::Mutex;

pub mod config;
pub mod repository;

#[derive(Clone, Debug)]
pub enum DB {
    Pool(MySqlPool<MySql>),
    Conn(Arc<Mutex<PoolConnection<MySql>>>),
}

#[async_trait]
pub trait MySQLContext: Component {
    async fn acquire(&self) -> Result<Arc<Mutex<PoolConnection<MySql>>>>;
}

#[async_trait]
impl Transaction for DB {
    async fn begin(&self) -> Result<DB> {
        match self {
            DB::Pool(_) => DB::begin(self.acquire().await?).await,
            DB::Conn(conn) => DB::begin(conn.clone()).await,
        }
    }

    async fn commit(&self) -> Result<DB> {
        match self {
            DB::Pool(_) => {
                log::warn!("{:?}", unexpected!("ignore DB:Pool commit."));
                Ok(self.clone())
            }
            DB::Conn(conn) => DB::commit(conn.clone()).await,
        }
    }
}
impl Drop for DB {
    fn drop(&mut self) {
        match self {
            DB::Pool(_) => (),
            DB::Conn(conn) => {
                let conn = conn.clone();
                tokio::task::spawn(DB::rollback(conn));
            }
        }
    }
}
#[async_trait]
impl MySQLContext for DB {
    async fn acquire(&self) -> Result<Arc<Mutex<PoolConnection<MySql>>>> {
        self.acquire().await
    }
}

impl DB {
    pub async fn new(config: MySQLConfig) -> Result<DB> {
        let db_pool = MySqlPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .acquire_timeout(config.connect_timeout)
            .connect(&config.url)
            .await
            .unwrap();
        Ok(DB::Pool(db_pool))
    }
    async fn begin(conn: Arc<Mutex<PoolConnection<MySql>>>) -> Result<DB> {
        <MySql as sqlx::Database>::TransactionManager::begin(&mut *conn.lock().await)
            .await
            .with_context(|| unexpected!("begin error"))?;
        Ok(DB::Conn(conn))
    }

    async fn commit(conn: Arc<Mutex<PoolConnection<MySql>>>) -> Result<DB> {
        <MySql as sqlx::Database>::TransactionManager::commit(&mut *conn.lock().await)
            .await
            .with_context(|| unexpected!("commit error"))?;
        Ok(DB::Conn(conn))
    }

    async fn rollback(conn: Arc<Mutex<PoolConnection<MySql>>>) -> Result<DB> {
        <MySql as sqlx::Database>::TransactionManager::start_rollback(&mut *conn.lock().await);
        Ok(DB::Conn(conn))
    }
    pub async fn acquire(&self) -> Result<Arc<Mutex<PoolConnection<MySql>>>> {
        match self {
            DB::Pool(pool) => Ok(Arc::new(Mutex::new(
                pool.clone()
                    .acquire()
                    .await
                    .with_context(|| unexpected!("acquire error"))?,
            ))),
            DB::Conn(conn) => Ok(conn.clone()),
        }
    }
}

#[async_trait]
impl UseContext for DB {
    type Context = DB;

    async fn context(&self) -> Result<Self::Context> {
        Ok(self.clone())
    }
}

pub mod dsl {
    pub use sea_query::*;
    #[must_use]
    pub fn alias(s: &str) -> Alias {
        Alias::new(s)
    }
    #[must_use]
    pub fn tbl(s: &str) -> impl Iden {
        alias(s)
    }
    #[must_use]
    pub fn col(s: &str) -> impl Iden {
        alias(s)
    }
    #[must_use]
    pub fn cond(s: &str) -> Expr {
        Expr::col(alias(s))
    }
}
