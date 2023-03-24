use crate::aws::ses::send_email;
use application::interface::gateway::mail::{MailGateway, SendEmailInput};
use application::interface::Component;
use async_trait::async_trait;
use kernel::Result;
use rusoto_sesv2::{Body, Content, Destination, EmailContent, Message, SendEmailRequest};

#[derive(Clone)]
pub struct SesMailGateway;

pub trait SesContext: Component {}

#[async_trait]
impl<Context: SesContext> MailGateway<Context> for SesMailGateway {
    async fn send_email(&self, _ctx: Context, input: SendEmailInput) -> Result<()> {
        let request = SendEmailRequest {
            content: EmailContent {
                simple: Some(Message {
                    body: Body {
                        text: Some(Content {
                            charset: Some("UTF-8".to_owned()),
                            data: input.body,
                        }),
                        ..Default::default()
                    },
                    subject: Content {
                        charset: Some("UTF-8".to_owned()),
                        data: input.subject,
                    },
                }),
                ..Default::default()
            },
            destination: Some(Destination {
                to_addresses: Some(vec![input.to_address.clone()]),
                ..Default::default()
            }),
            from_email_address: Some(input.from_address.clone()),
            ..Default::default()
        };
        send_email(request).await
    }
}
