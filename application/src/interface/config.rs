use derive_builder::Builder;

#[derive(Clone, Debug, Default, new, Builder)]
#[builder(setter(into))]
pub struct Config {
    pub(crate) auth: Auth,
    pub(crate) system: System,
}
#[derive(Clone, Debug, Default, new, Builder)]
#[builder(setter(into))]
pub struct System {
    pub(crate) mail_domain: String,
}

#[derive(Clone, Debug, Default, new, Builder)]
#[builder(setter(into))]
pub struct Auth {
    pub(crate) pepper: String,
    pub(crate) stretch_count: i64,
}
