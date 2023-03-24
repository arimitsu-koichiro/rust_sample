#[macro_use]
extern crate derive_new;

pub mod entity;

pub mod error;

pub use anyhow::Result;

pub mod build_info {
    use std::env;
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
    #[must_use]
    pub fn get_version() -> Option<String> {
        GIT_COMMIT_HASH
            .map(std::string::ToString::to_string)
            .or_else(|| env::var("GIT_COMMIT_HASH").ok())
    }
}
