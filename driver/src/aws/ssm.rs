use anyhow::bail;
use aws_sdk_ssm as ssm;
use helper::env::get_var;
use kernel::unexpected;
use kernel::Result;

use crate::aws::init_config_loader;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::env;

pub async fn load_from_ssm(path: String) -> Result<()> {
    let ssm_envs = get_parameters_by_path(&path, Some(true), Some(false)).await?;
    for (key, value) in &ssm_envs {
        env::set_var(key.replace(&path, ""), value);
    }
    Ok(())
}

pub async fn get_parameter(name: &str, with_decryption: Option<bool>) -> Result<Option<String>> {
    match SSM_CLIENT
        .get_parameter()
        .name(name)
        .set_with_decryption(with_decryption)
        .send()
        .await
    {
        Ok(result) => match result.parameter() {
            Some(param) => Ok(param.value().map(Into::into)),
            None => Ok(None),
        },
        Err(e) => bail!(unexpected!("ssm get_parameter error {:?}", e)),
    }
}

pub async fn get_parameters_by_path(
    path: &str,
    with_decryption: Option<bool>,
    recursive: Option<bool>,
) -> Result<HashMap<String, String>> {
    match SSM_CLIENT
        .get_parameters_by_path()
        .path(path)
        .set_with_decryption(with_decryption)
        .set_recursive(recursive)
        .set_parameter_filters(None)
        .set_max_results(None)
        .set_next_token(None)
        .send()
        .await
    {
        Ok(result) => match result.parameters() {
            Some(params) => {
                let mut result: HashMap<String, String> = HashMap::new();
                for param in params {
                    if let (Some(name), Some(value)) = (param.name(), param.value()) {
                        result.insert(name.into(), value.into());
                    }
                }
                Ok(result)
            }
            None => Ok(HashMap::new()),
        },
        Err(e) => bail!(unexpected!("ssm get_parameter error. {:?}", e)),
    }
}

static SSM_CLIENT: Lazy<ssm::Client> = Lazy::new(|| {
    let config = futures::executor::block_on(
        init_config_loader(get_var::<String>("SSM_ENDPOINT_URL").ok()).load(),
    );
    ssm::Client::new(&config)
});
