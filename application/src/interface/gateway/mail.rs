use crate::interface::Component;
use async_trait::async_trait;
use blanket::blanket;
use kernel::Result;
#[cfg(test)]
use mockall::mock;

#[async_trait]
#[blanket(derive(Arc))]
pub trait MailGateway<Context>: Component {
    async fn send_email(&self, ctx: Context, input: SendEmailInput) -> Result<()>;
}

pub trait UseMailGateway<Context> {
    type Gateway: MailGateway<Context>;
    fn mail_gateway(&self) -> Self::Gateway;
}

#[derive(new)]
pub struct SendEmailInput {
    pub from_address: String,
    pub to_address: String,
    pub subject: String,
    pub body: String,
}

#[cfg(test)]
mock! {
    pub MailGateway{}
    impl Clone for MailGateway {
        fn clone(&self) -> Self;
    }
    #[async_trait]
    impl MailGateway<()> for MailGateway {
        async fn send_email(&self, ctx: (), input: SendEmailInput) -> Result<()>;
    }
}
