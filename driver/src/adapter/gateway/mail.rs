use crate::aws::ses::send_email;
use application::interface::gateway::mail::{MailGateway, SendEmailInput};
use application::interface::Component;
use async_trait::async_trait;
use helper::validation::Validation;
use kernel::Result;

#[derive(Clone)]
pub struct SesMailGateway;

pub trait SesContext: Component {}

#[async_trait]
impl<Context: SesContext> MailGateway<Context> for SesMailGateway {
    async fn send_email(&self, _ctx: Context, input: SendEmailInput) -> Result<()> {
        let input = input.validate()?;
        send_email(input).await
    }
}
