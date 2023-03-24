use crate::interface::Component;
use async_trait::async_trait;
use blanket::blanket;
use kernel::entity::Account;
use kernel::Result;

#[async_trait]
#[blanket(derive(Arc))]
pub trait AccountRepository<Context>: Component {
    async fn get(&self, ctx: Context, id: String) -> Result<Option<Account>>;
    async fn create(&self, ctx: Context, account: NewAccount) -> Result<Account>;
}

pub trait UseAccountRepository<Context> {
    type AccountRepository: AccountRepository<Context>;
    fn account_repository(&self) -> Self::AccountRepository;
}

#[derive(Clone, Debug, new)]
pub struct NewAccount {
    pub id: String,
    pub name: String,
    pub display_name: String,
}
