use crate::aws::with_profile;
use anyhow::{Context as _, Result};
use kernel::unexpected;
use rusoto_core::{HttpClient, Region};
pub use rusoto_sesv2::*;

pub async fn send_email(request: SendEmailRequest) -> Result<()> {
    let client = get_client()?;
    match client.send_email(request).await {
        Ok(_) => Ok(()),
        Err(e) => Err(e).with_context(|| unexpected!("send_email error")),
    }
}

fn get_client() -> Result<SesV2Client> {
    with_profile(|x| match x {
        Some(provider) => Ok(SesV2Client::new_with(
            HttpClient::new()?,
            provider,
            Region::default(),
        )),
        _ => Ok(SesV2Client::new(Region::default())),
    })
}
