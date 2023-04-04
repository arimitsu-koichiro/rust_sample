use anyhow::Result;

use sha2;
use sha2::Digest;

pub fn stretch_password(origin: &str, salt: &str, pepper: &str, count: i64) -> Result<String> {
    let mut output = format!("{origin}:{salt}:{pepper}");
    for _ in 0..count {
        let mut hash = sha2::Sha512::new();
        hash.update(output.as_bytes());
        output = format!("{:x}", hash.finalize());
    }
    Ok(output)
}
