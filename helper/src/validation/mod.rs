use validator::Validate;

pub trait Validation: Sized {
    fn validate(self) -> anyhow::Result<Self>;
}

impl<T: Validate> Validation for T {
    fn validate(self) -> anyhow::Result<Self> {
        Validate::validate(&self)?;
        Ok(self)
    }
}
