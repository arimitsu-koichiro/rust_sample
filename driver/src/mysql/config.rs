use derive_new::new;
use std::time::Duration;

#[derive(Clone, new)]
pub struct MySQLConfig {
    pub(crate) url: String,
    pub(crate) min_connections: u32,
    pub(crate) max_connections: u32,
    pub(crate) connect_timeout: Duration,
    pub(crate) idle_timeout: Duration,
    pub(crate) max_lifetime: Duration,
}
