use derive_new::new;
use std::net::SocketAddr;

#[derive(Clone, new)]
pub struct Config {
    pub(crate) bind_address: SocketAddr,
}
