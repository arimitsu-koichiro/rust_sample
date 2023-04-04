#[must_use]
pub fn new_v4() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}

pub trait ToBase62 {
    fn to_base62(&self) -> String;
}

// return 21 or 22 length string
impl ToBase62 for uuid::Uuid {
    fn to_base62(&self) -> String {
        base62::encode(self.as_u128())
    }
}

#[must_use]
pub fn new_ulid() -> ulid::Ulid {
    ulid::Ulid::new()
}

impl ToBase62 for ulid::Ulid {
    fn to_base62(&self) -> String {
        base62::encode(*self)
    }
}
