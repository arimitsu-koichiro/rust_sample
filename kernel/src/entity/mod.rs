use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate, Debug, Clone, new)]
pub struct ProvisionalSession {
    pub code: String,
    pub authentication: ProvisionalAuthentication,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, new)]
pub struct Session {
    pub id: String,
    #[validate]
    pub account: Account,
    pub create_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, new)]
pub struct Account {
    pub id: String,
    #[validate(length(max = 100))]
    pub name: String,
    #[validate(length(max = 100))]
    pub display_name: String,
    pub create_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, new)]
pub struct ProvisionalAuthentication {
    #[validate(email)]
    pub mail: String,
    pub salt: String,
    #[validate(length(equal = 128))]
    pub password_hash: String,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, new)]
pub struct Authentication {
    pub account_id: String,
    #[validate(email)]
    pub mail: String,
    pub salt: String,
    #[validate(length(equal = 128))]
    pub password_hash: String,
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, new)]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub create_time: DateTime<Utc>,
}
