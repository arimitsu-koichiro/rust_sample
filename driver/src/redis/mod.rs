use crate::redis::config::RedisConfig;
use application::interface::Component;
use async_trait::async_trait;
use bb8_redis::bb8;
pub use bb8_redis::bb8::Pool as RedisPool;
pub use bb8_redis::bb8::PooledConnection as PooledRedisConnection;
pub use bb8_redis::RedisConnectionManager;
pub type RedisConnection = <RedisConnectionManager as bb8::ManageConnection>::Connection;
use kernel::Result;
use marker_blanket::marker_blanket;

pub mod config;
pub mod gateway;
pub mod repository;

#[derive(Clone, Debug)]
pub enum Redis {
    Pool {
        primary: RedisPool<RedisConnectionManager>,
        reader: RedisPool<RedisConnectionManager>,
    },
}

#[async_trait]
pub trait RedisPrimaryContext: Component {
    async fn primary(&self) -> Result<PooledRedisConnection<RedisConnectionManager>>;
}

#[async_trait]
pub trait RedisReaderContext: Component {
    async fn reader(&self) -> Result<PooledRedisConnection<RedisConnectionManager>>;
    async fn subscribe_connection(&self) -> Result<RedisConnection>;
}

#[marker_blanket]
pub trait RedisContext: RedisPrimaryContext + RedisReaderContext {}

impl Redis {
    pub async fn new(config: RedisConfig) -> Result<Redis> {
        let manager = RedisConnectionManager::new(config.primary_url.clone()).unwrap();
        let primary = RedisPool::builder()
            .min_idle(config.min_idle)
            .max_size(config.max_size)
            .build(manager)
            .await
            .unwrap();
        let manager = RedisConnectionManager::new(config.reader_url.clone()).unwrap();
        let reader = RedisPool::builder()
            .min_idle(config.min_idle)
            .max_size(config.max_size)
            .build(manager)
            .await
            .unwrap();
        Ok(Redis::Pool { primary, reader })
    }
}

#[async_trait]
impl RedisPrimaryContext for Redis {
    async fn primary(&self) -> Result<PooledRedisConnection<RedisConnectionManager>> {
        match self {
            Redis::Pool { primary, reader: _ } => Ok(primary.get().await?),
        }
    }
}

#[async_trait]
impl RedisReaderContext for Redis {
    async fn reader(&self) -> Result<PooledRedisConnection<RedisConnectionManager>> {
        match self {
            Redis::Pool { primary: _, reader } => Ok(reader.get().await?),
        }
    }

    async fn subscribe_connection(&self) -> Result<RedisConnection> {
        match self {
            Redis::Pool { primary: _, reader } => Ok(reader.dedicated_connection().await?),
        }
    }
}

#[must_use]
pub fn compose_key(name_space: &str, key: &str) -> String {
    format!("{name_space}:{key}")
}

// pub struct MockRedisClient {
//     storage: Arc<Mutex<HashMap<String, Vec<u8>>>>,
// }
// use std::sync::{Arc, Mutex};
// use std::collections::HashMap;
// use crate::error::map_poison_err;
// impl RedisClient for MockRedisClient {
//     fn set(&self, name_space: &str, key: &str, value: &[u8]) -> Result<()> {
//         let storage = self.storage.clone();
//         let mut storage = storage.lock().map_err(map_poison_err)?;
//         storage.insert(compose_key(name_space, key), value.to_vec());
//         Ok(())
//     }
//     fn get(&self, name_space: &str, key: &str) -> Result<Option<Vec<u8>>> {
//         let storage = self.storage.clone();
//         let storage = storage.lock().map_err(map_poison_err)?;
//         let result = storage
//             .get(&compose_key(name_space, key))
//             .map(|r| r.to_owned());
//         Ok(result)
//     }
//     fn delete(&self, name_space: &str, key: &str) -> Result<usize> {
//         let storage = self.storage.clone();
//         let mut storage = storage.lock().map_err(map_poison_err)?;
//         let opt = storage.remove(&compose_key(name_space, key));
//         Ok(if opt.is_some() { 1 } else { 0 })
//     }
// }
