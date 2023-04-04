use aws_config::ConfigLoader;
use helper::env::get_var;

pub mod s3;
pub mod ses;
pub mod ssm;

pub(crate) fn init_config_loader(endpoint_url: Option<String>) -> ConfigLoader {
    let mut loader = aws_config::from_env();
    if let Ok(profile) = get_var::<String>("AWS_PROFILE") {
        loader = loader.profile_name(profile);
    }
    if let Some(endpoint_url) = endpoint_url {
        loader = loader.endpoint_url(endpoint_url);
    }
    loader
}
