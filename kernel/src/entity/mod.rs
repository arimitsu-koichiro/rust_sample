use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct ProvisionalSession {
    pub code: String,
    pub mail: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct Session {
    pub id: String,
    pub account: Account,
    pub create_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub create_time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct Authentication {
    pub account_id: String,
    pub mail: String,
    pub salt: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, new)]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub create_time: DateTime<Utc>,
}
