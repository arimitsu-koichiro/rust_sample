use derive_new::new;

#[derive(Clone, new)]
pub struct Config {
    pub(crate) primary_url: String,
    pub(crate) reader_url: String,
    pub(crate) min_idle: Option<u32>,
    pub(crate) max_size: u32,
}
