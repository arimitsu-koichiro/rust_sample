use crate::aws::init_config_loader;
use anyhow::{Context as _, Result};
use application::interface::gateway::mail::SendEmailInput;
use aws_sdk_ses as ses;
use aws_sdk_ses::types::{Body, Content, Destination, Message};
use helper::env::get_var;
use kernel::unexpected;
use once_cell::sync::Lazy;

pub async fn send_email(request: SendEmailInput) -> Result<()> {
    let output = SES_CLIENT
        .send_email()
        .source(request.from_address)
        .destination(
            Destination::builder()
                .to_addresses(request.to_address)
                .build(),
        )
        .message(
            Message::builder()
                .subject(Content::builder().data(request.subject).build())
                .body(
                    Body::builder()
                        .text(Content::builder().data(request.body).build())
                        .build(),
                )
                .build(),
        )
        .send()
        .await;
    match output {
        Ok(_) => Ok(()),
        Err(e) => Err(e).with_context(|| unexpected!("send_email error")),
    }
}

static SES_CLIENT: Lazy<ses::Client> = Lazy::new(|| {
    let config = futures::executor::block_on(
        init_config_loader(get_var::<String>("SES_ENDPOINT_URL").ok()).load(),
    );
    ses::Client::new(&config)
});
