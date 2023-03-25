use crate::env::get_var;
use anyhow::Result;
use sha2;
use sha2::Digest;

#[must_use]
pub fn stretch(origin: &str, salt: &str, pepper: &str, count: i64) -> String {
    let mut output = format!("{origin}:{salt}:{pepper}");
    for _ in 0..count {
        let mut hash = sha2::Sha512::new();
        hash.update(output.as_bytes());
        output = format!("{:x}", hash.finalize());
    }
    output
}

pub fn stretch_password(password: &str, salt: &str) -> Result<String> {
    let pepper = get_var::<String>("AUTH_PEPPER")?;
    let stretch_count: i64 = get_var("AUTH_STRETCH_COUNT")?;
    Ok(stretch(password, salt, &pepper, stretch_count))
}
