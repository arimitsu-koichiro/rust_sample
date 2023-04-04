pub mod account {
    use crate::mysql::dsl::{col, cond, tbl, Expr, MysqlQueryBuilder, Query};
    use crate::mysql::MySQLContext;
    use anyhow::Context;
    use chrono::{DateTime, Utc};
    use kernel::Result;
    use kernel::{entity, unexpected};
    use sea_query_binder::{SqlxBinder, SqlxValues};

    pub async fn get(db: impl MySQLContext, id: String) -> Result<Option<entity::Account>> {
        let (query, values) = Query::select()
            .expr(Expr::asterisk())
            .from(tbl("account"))
            .and_where(cond("id").eq(id))
            .build_sqlx(MysqlQueryBuilder);
        query_account_with(db, query, values).await
    }

    pub async fn create(
        db: impl MySQLContext,
        new_account: entity::Account,
    ) -> Result<entity::Account> {
        let (query, values) = Query::insert()
            .into_table(tbl("account"))
            .columns(vec![
                col("id"),
                col("name"),
                col("display_name"),
                col("create_time"),
            ])
            .values(vec![
                new_account.id.clone().into(),
                new_account.name.clone().into(),
                new_account.display_name.clone().into(),
                new_account.create_time.into(),
            ])?
            .build_sqlx(MysqlQueryBuilder);
        match sqlx::query_with(&query, values)
            .execute(&mut *db.acquire().await?.lock().await)
            .await
        {
            Err(err) => Err(err).with_context(|| unexpected!("account create error")),
            Ok(_) => Ok(entity::Account::new(
                new_account.id,
                new_account.name,
                new_account.display_name,
                new_account.create_time,
            )),
        }
    }
    async fn query_account_with(
        db: impl MySQLContext,
        query: String,
        values: SqlxValues,
    ) -> Result<Option<entity::Account>> {
        match sqlx::query_as_with::<_, Account, _>(&query, values)
            .fetch_optional(&mut *db.acquire().await?.lock().await)
            .await
        {
            Err(err) => Err(err).with_context(|| unexpected!("account query_account_with error")),
            Ok(res) => Ok(res.map(entity::Account::from)),
        }
    }

    #[derive(sqlx::FromRow, Debug, Clone)]
    pub struct Account {
        pub id: String,
        pub name: String,
        pub display_name: String,
        pub create_time: DateTime<Utc>,
    }

    impl From<Account> for entity::Account {
        fn from(record: Account) -> Self {
            entity::Account::new(
                record.id,
                record.name,
                record.display_name,
                record.create_time,
            )
        }
    }
}

pub mod authentication {
    use crate::mysql::dsl::{col, cond, tbl, Expr, MysqlQueryBuilder, Query};
    use crate::mysql::MySQLContext;
    use anyhow::Context;
    use application::interface::repository::authentication::UpdatePassword;
    use kernel::Result;
    use kernel::{entity, unexpected};
    use sea_query_binder::SqlxBinder;

    pub async fn get_by_mail(
        db: impl MySQLContext,
        mail: String,
    ) -> Result<Option<entity::Authentication>> {
        let (query, values) = Query::select()
            .expr(Expr::asterisk())
            .from(tbl("authentication"))
            .and_where(cond("mail").eq(mail))
            .build_sqlx(MysqlQueryBuilder);
        match sqlx::query_as_with::<_, Authentication, _>(&query, values)
            .fetch_optional(&mut *db.acquire().await?.lock().await)
            .await
        {
            Err(err) => Err(err).with_context(|| unexpected!("authentication get_by_mail error")),
            Ok(res) => Ok(res.map(entity::Authentication::from)),
        }
    }
    pub async fn create(
        db: impl MySQLContext,
        new_authentication: entity::Authentication,
    ) -> Result<()> {
        let (query, values) = Query::insert()
            .into_table(tbl("authentication"))
            .columns(vec![
                col("account_id"),
                col("mail"),
                col("salt"),
                col("password"),
            ])
            .values(vec![
                new_authentication.account_id.into(),
                new_authentication.mail.into(),
                new_authentication.salt.into(),
                new_authentication.password_hash.into(),
            ])?
            .build_sqlx(MysqlQueryBuilder);
        match sqlx::query_with(&query, values)
            .execute(&mut *db.acquire().await?.lock().await)
            .await
        {
            Err(err) => Err(err).with_context(|| unexpected!("authentication create error")),
            Ok(_) => Ok(()),
        }
    }
    pub async fn update_password(db: impl MySQLContext, updated: UpdatePassword) -> Result<()> {
        let (query, values) = Query::update()
            .table(tbl("authentication"))
            .values(vec![(col("password"), updated.password.into())])
            .and_where(cond("account_id").eq(updated.account_id))
            .and_where(cond("mail").eq(updated.mail))
            .build_sqlx(MysqlQueryBuilder);
        match sqlx::query_with(&query, values)
            .execute(&mut *db.acquire().await?.lock().await)
            .await
        {
            Err(err) => Err(err).with_context(|| unexpected!("update_password error")),
            Ok(_) => Ok(()),
        }
    }

    #[derive(sqlx::FromRow, Debug, Clone)]
    pub struct Authentication {
        pub account_id: String,
        pub mail: String,
        pub salt: String,
        pub password: String,
    }

    impl From<Authentication> for entity::Authentication {
        fn from(record: Authentication) -> Self {
            entity::Authentication::new(
                record.account_id,
                record.mail,
                record.salt,
                record.password,
            )
        }
    }
}

pub mod comment {
    use crate::mysql::dsl::{col, cond, tbl, Expr, MysqlQueryBuilder, Query};
    use crate::mysql::MySQLContext;
    use anyhow::Context;
    use chrono::{DateTime, Utc};
    use helper::time::current_time;
    use kernel::Result;
    use kernel::{entity, unexpected};
    use sea_query_binder::SqlxBinder;

    pub async fn get(db: impl MySQLContext, id: String) -> Result<Option<entity::Comment>> {
        let (query, values) = Query::select()
            .expr(Expr::asterisk())
            .from(tbl("comment"))
            .and_where(cond("id").eq(id))
            .build_sqlx(MysqlQueryBuilder);
        match sqlx::query_as_with::<_, Comment, _>(&query, values)
            .fetch_optional(&mut *db.acquire().await?.lock().await)
            .await
        {
            Err(err) => Err(err).with_context(|| unexpected!("comment get error")),
            Ok(res) => Ok(res.map(entity::Comment::from)),
        }
    }

    pub async fn put(
        db: impl MySQLContext,
        id: String,
        body: String,
    ) -> Result<Option<entity::Comment>> {
        let now = current_time();
        let (query, values) = Query::insert()
            .into_table(tbl("comment"))
            .columns(vec![col("id"), col("body"), col("create_time")])
            .values(vec![id.clone().into(), body.clone().into(), now.into()])?
            .build_sqlx(MysqlQueryBuilder);
        match sqlx::query_with(&query, values)
            .execute(&mut *db.acquire().await?.lock().await)
            .await
        {
            Err(err) => Err(err).with_context(|| unexpected!("comment put error")),
            Ok(_) => Ok(Some(entity::Comment::new(id, body, now))),
        }
    }

    #[derive(sqlx::FromRow, Debug, Clone)]
    pub struct Comment {
        pub id: String,
        pub body: String,
        pub create_time: DateTime<Utc>,
    }
    impl From<Comment> for entity::Comment {
        fn from(record: Comment) -> Self {
            entity::Comment::new(record.id, record.body, record.create_time)
        }
    }
}
