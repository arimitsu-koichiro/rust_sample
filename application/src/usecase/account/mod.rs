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
        let ctx = self.deps.context().await?;
        if input.id == "me" {
            let Some(session_id) = input.session_id else {
                bail!(forbidden!("required session."))
            };
            let Some(session) = self.deps.session_repository().get(ctx.clone(), session_id).await? else {
                bail!(forbidden!("required session."))
            };
            return Ok(GetAccountOutput {
                account: Some(session.account),
            });
        }
        let account = self.deps.account_repository().get(ctx, input.id).await?;
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
    use crate::interface::repository::account::{MockAccountRepository, UseAccountRepository};
    use crate::interface::repository::session::{MockSessionRepository, UseSessionRepository};
    use std::sync::Arc;

    use crate::interface::UseContext;
    use crate::usecase::account::{GetAccountInput, GetAccountUseCase};
    use crate::usecase::UseCase;
    use async_trait::async_trait;

    use helper::time::current_time;
    use kernel::entity::Account;
    use kernel::Result;
    use mockall::predicate;

    #[derive(Clone)]
    struct TestMods {
        mock_account_repo: Arc<MockAccountRepository>,
        mock_session_repo: Arc<MockSessionRepository>,
    }

    #[async_trait]
    impl UseContext for TestMods {
        type Context = ();

        async fn context(&self) -> Result<Self::Context> {
            Ok(())
        }
    }

    impl UseAccountRepository<()> for TestMods {
        type AccountRepository = Arc<MockAccountRepository>;

        fn account_repository(&self) -> Self::AccountRepository {
            self.mock_account_repo.clone()
        }
    }
    impl UseSessionRepository<()> for TestMods {
        type SessionRepository = Arc<MockSessionRepository>;

        fn session_repository(&self) -> Self::SessionRepository {
            self.mock_session_repo.clone()
        }
    }

    #[tokio::test]
    async fn get_account() {
        let mut mock_account_repo = MockAccountRepository::default();
        mock_account_repo
            .expect_get()
            .with(predicate::eq(()), predicate::eq("id".to_string()))
            .return_once(|_, id| {
                Ok(Some(Account::new(
                    id,
                    "name".to_string(),
                    "display_name".to_string(),
                    current_time(),
                )))
            });
        let mods = TestMods {
            mock_account_repo: Arc::new(mock_account_repo),
            mock_session_repo: Arc::new(MockSessionRepository::default()),
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
    #[tokio::test]
    async fn get_account_with_me() {
        let mock_account_repo = MockAccountRepository::default();
        let mut mock_session_repo = MockSessionRepository::default();
        let now = current_time();
        mock_session_repo
            .expect_get()
            .with(predicate::eq(()), predicate::eq("session_id".to_string()))
            .return_once(move |_, session_id| {
                Ok(Some(kernel::entity::Session::new(
                    session_id,
                    Account::new(
                        "account_id".to_string(),
                        "name".to_string(),
                        "display_name".to_string(),
                        now,
                    ),
                    now,
                )))
            });
        let mods = TestMods {
            mock_account_repo: Arc::new(mock_account_repo),
            mock_session_repo: Arc::new(mock_session_repo),
        };
        let interactor = GetAccountUseCase::new(mods);
        let input = GetAccountInput {
            id: "me".to_string(),
            session_id: Some("session_id".to_string()),
        };
        let output = interactor.handle(input).await;
        let account = output.unwrap().account.unwrap();
        assert_eq!(account.id, "account_id".to_string());
    }
}
