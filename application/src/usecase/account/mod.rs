use crate::interface::repository::account::{AccountRepository, UseAccountRepository};
use crate::interface::repository::session::{SessionRepository, UseSessionRepository};
use crate::interface::Component;
use crate::interface::UseContext;
use crate::usecase::UseCase;
use anyhow::{bail, Result};
use async_trait::async_trait;
use kernel::entity::Account;
use kernel::forbidden;
use std::marker::PhantomData;
use trait_set::trait_set;

#[derive(Clone, new)]
pub struct GetAccountUseCase<C, Deps> {
    pub(crate) deps: Deps,
    _c: PhantomData<C>,
}

trait_set! {
    pub trait GetAccountUseCaseDeps<C: Component> = Component + UseContext<Context = C>
    + UseAccountRepository<C>
    + UseSessionRepository<C>
    ;
}

#[async_trait]
impl<C, Deps> UseCase<GetAccountInput, GetAccountOutput> for GetAccountUseCase<C, Deps>
where
    C: Component,
    Deps: GetAccountUseCaseDeps<C>,
{
    async fn handle(&self, input: GetAccountInput) -> Result<GetAccountOutput> {
        let db = self.deps.context().await?;
        if input.id == "me" {
            let Some(session_id) = input.session_id else {
                bail!(forbidden!("required session."))
            };
            let Some(session) = self.deps.session_repository().get(db.clone(), session_id).await? else {
                bail!(forbidden!("required session."))
            };
            return Ok(GetAccountOutput {
                account: Some(session.account),
            });
        }
        let account = self.deps.account_repository().get(db, input.id).await?;
        Ok(GetAccountOutput::new(account))
    }
}

#[derive(new)]
pub struct GetAccountInput {
    pub(crate) id: String,
    pub(crate) session_id: Option<String>,
}

#[derive(new)]
pub struct GetAccountOutput {
    pub account: Option<Account>,
}

#[cfg(test)]
mod tests {
    use crate::interface::repository::account::{
        AccountRepository, NewAccount, UseAccountRepository,
    };
    use crate::interface::repository::session::{SessionRepository, UseSessionRepository};
    use std::sync::Arc;

    use crate::interface::UseContext;
    use crate::usecase::account::{GetAccountInput, GetAccountUseCase};
    use crate::usecase::UseCase;
    use async_trait::async_trait;
    use chrono::DateTime;
    use kernel::entity::{Account, ProvisionalSession, Session};
    use kernel::Result;
    use mockall::{mock, predicate};

    #[derive(Clone)]
    struct TestMods {
        mock_account_repo: Arc<MockTestAccountRepository>,
        mock_session_repo: Arc<MockTestSessionRepository>,
    }

    #[async_trait]
    impl UseContext for TestMods {
        type Context = ();

        async fn context(&self) -> Result<Self::Context> {
            Ok(())
        }
    }
    mock! {
        pub TestAccountRepository {
        }
        impl Clone for TestAccountRepository {
            fn clone(&self) -> Self;
        }
        #[async_trait]
        impl AccountRepository<()> for TestAccountRepository {
            async fn get(&self, ctx: (), id: String) -> Result<Option<Account>>;
            async fn create(&self, ctx: (), account: NewAccount) -> Result<Account>;
        }
    }
    mock! {
        pub TestSessionRepository {
        }
        impl Clone for TestSessionRepository {
            fn clone(&self) -> Self;
        }
        #[async_trait]
        impl SessionRepository<()> for TestSessionRepository {
            async fn set_provisional_session(
                &self,
                ctx: (),
                session: ProvisionalSession,
            ) -> Result<()>;
            async fn get_provisional_session(
                &self,
                ctx: (),
                id: String,
            ) -> Result<Option<ProvisionalSession>>;
            async fn set(&self, ctx: (), session: Session) -> Result<()>;
            async fn get(&self, ctx: (), id: String) -> Result<Option<Session>>;
            async fn delete(&self, ctx: (), id: String) -> Result<()>;
        }
    }

    impl UseAccountRepository<()> for TestMods {
        type AccountRepository = Arc<MockTestAccountRepository>;

        fn account_repository(&self) -> Self::AccountRepository {
            self.mock_account_repo.clone()
        }
    }
    impl UseSessionRepository<()> for TestMods {
        type SessionRepository = Arc<MockTestSessionRepository>;

        fn session_repository(&self) -> Self::SessionRepository {
            self.mock_session_repo.clone()
        }
    }

    #[tokio::test]
    async fn get_account() {
        let mut mock_account_repo = MockTestAccountRepository::default();
        mock_account_repo
            .expect_get()
            .with(predicate::eq(()), predicate::eq("id".to_string()))
            .return_once(|_, id| {
                Ok(Some(Account::new(
                    id,
                    "name".to_string(),
                    "display_name".to_string(),
                    DateTime::default(),
                )))
            });
        let mods = TestMods {
            mock_account_repo: Arc::new(mock_account_repo),
            mock_session_repo: Arc::new(MockTestSessionRepository::default()),
        };
        let interactor = GetAccountUseCase::new(mods);
        let output = interactor
            .handle(GetAccountInput {
                id: "id".to_string(),
                session_id: None,
            })
            .await;
        assert_eq!(output.unwrap().account.unwrap().id, "id".to_string());
    }
}
