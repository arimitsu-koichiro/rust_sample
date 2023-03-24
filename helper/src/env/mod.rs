use anyhow::{bail, Context as _, Result};
use std::env;
use std::fmt::Debug;
use std::str::FromStr;

pub fn get_var<T: FromStr>(name: &str) -> Result<T>
where
    <T as FromStr>::Err: Debug,
{
    let val = env::var(name).with_context(|| format!("get_var error {name}"))?;
    match val.parse::<T>() {
        Ok(val) => Ok(val),
        Err(e) => bail!("parse error {:?}", e),
    }
}

pub fn get_var_or<T: FromStr>(name: &str, d: T) -> T
where
    <T as FromStr>::Err: Debug,
{
    get_var(name).unwrap_or(d)
}

pub fn var_is_opt<T: FromStr + PartialEq>(name: &str, value: Option<T>) -> bool
where
    <T as FromStr>::Err: Debug,
{
    get_var_opt(name) == value
}

pub fn var_is<T: FromStr + PartialEq>(name: &str, value: T) -> bool
where
    <T as FromStr>::Err: Debug,
{
    var_is_opt(name, Some(value))
}

#[must_use]
pub fn get_var_opt<T: FromStr>(name: &str) -> Option<T>
where
    <T as FromStr>::Err: Debug,
{
    get_var(name).ok()
}
