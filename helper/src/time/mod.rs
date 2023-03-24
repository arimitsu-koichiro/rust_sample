use chrono::{DateTime, Utc};

#[must_use]
pub fn current_time() -> DateTime<Utc> {
    Utc::now()
}
