#[macro_use]
extern crate derive_new;

pub mod entity;

pub mod error;

pub use error::Error;

pub type Result<T, E = anyhow::Error> = anyhow::Result<T, E>;

pub mod build_info {
    use std::env;
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
    #[must_use]
    pub fn git_commit_hash() -> Option<String> {
        GIT_COMMIT_HASH
            .map(std::string::ToString::to_string)
            .or_else(|| env::var("GIT_COMMIT_HASH").ok())
    }
    #[must_use]
    pub fn build_time_utc() -> String {
        BUILT_TIME_UTC.to_string()
    }
}
