use crate::aws::with_profile;
use anyhow::{bail, Result};

use kernel::unexpected;
use rusoto_core::{HttpClient, Region};
use rusoto_ssm::{Ssm, SsmClient};
use std::collections::HashMap;
use std::env;

fn get_client() -> SsmClient {
    with_profile(|x| match x {
        Some(provider) => {
            SsmClient::new_with(HttpClient::new().unwrap(), provider, Region::default())
        }
        _ => SsmClient::new(Region::default()),
    })
}

pub async fn load_from_ssm(path: String) -> anyhow::Result<()> {
    let ssm_envs = get_parameters_by_path(&path, Some(true), Some(false)).await?;
    for (key, value) in &ssm_envs {
        env::set_var(key.replace(&path, ""), value);
    }
    Ok(())
}

#[allow(dead_code)]
pub async fn get_parameter(name: &str, with_decryption: Option<bool>) -> Result<Option<String>> {
    let client = get_client();
    match client
        .get_parameter(rusoto_ssm::GetParameterRequest {
            name: name.to_string(),
            with_decryption,
        })
        .await
    {
        Ok(result) => match result.parameter {
            Some(param) => Ok(param.value),
            None => Ok(None),
        },
        Err(e) => bail!(unexpected!("ssm get_parameter error {:?}", e)),
    }
}

#[allow(dead_code)]
pub async fn get_parameters_by_path(
    path: &str,
    with_decryption: Option<bool>,
    recursive: Option<bool>,
) -> Result<HashMap<String, String>> {
    let client = get_client();
    match client
        .get_parameters_by_path(rusoto_ssm::GetParametersByPathRequest {
            path: path.to_string(),
            recursive,
            with_decryption,
            parameter_filters: None,
            max_results: None,
            next_token: None,
        })
        .await
    {
        Ok(result) => match result.parameters {
            Some(params) => {
                let mut result = HashMap::new();
                for param in params {
                    if let (Some(name), Some(value)) = (param.name, param.value) {
                        result.insert(name, value);
                    }
                }
                Ok(result)
            }
            None => Ok(HashMap::new()),
        },
        Err(e) => bail!(unexpected!("ssm get_parameter error. {:?}", e)),
    }
}
