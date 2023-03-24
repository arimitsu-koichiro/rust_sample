use helper::env::get_var_opt;
use rusoto_core::credential::ProfileProvider;
use rusoto_core::HttpClient;

pub mod s3;
pub mod ses;
pub mod ssm;

pub fn with_profile<F, A>(f: F) -> A
where
    F: FnOnce(Option<ProfileProvider>) -> A,
{
    match get_var_opt::<String>("AWS_PROFILE") {
        Some(profile) => {
            let mut provider = ProfileProvider::new().unwrap();
            provider.set_profile(profile);
            HttpClient::new().unwrap();
            f(Some(provider))
        }
        _ => f(None),
    }
}
