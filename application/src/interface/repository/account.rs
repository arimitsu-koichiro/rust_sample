use crate::interface::Component;
use async_trait::async_trait;
use blanket::blanket;
use kernel::entity::Account;
use kernel::Result;
#[cfg(test)]
use mockall::mock;

#[async_trait]
#[blanket(derive(Arc))]
pub trait AccountRepository<Context>: Component {
    async fn get(&self, ctx: Context, id: String) -> Result<Option<Account>>;
    async fn create(&self, ctx: Context, account: Account) -> Result<Account>;
}

pub trait UseAccountRepository<Context> {
    type AccountRepository: AccountRepository<Context>;
    fn account_repository(&self) -> Self::AccountRepository;
}

#[cfg(test)]
mock! {
    pub AccountRepository{}
    impl Clone for AccountRepository {
        fn clone(&self) -> Self;
    }
    #[async_trait]
    impl AccountRepository<()> for AccountRepository {
        async fn get(&self, ctx: (), id: String) -> Result<Option<Account>>;
        async fn create(&self, ctx: (), account: Account) -> Result<Account>;
    }
}
