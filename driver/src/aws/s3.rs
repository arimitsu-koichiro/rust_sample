use crate::aws::init_config_loader;
use aws_sdk_s3 as s3;
use aws_sdk_s3::types::Bucket;
use helper::env::get_var;
use kernel::Result;
use once_cell::sync::Lazy;

pub async fn ls() -> Result<Option<Vec<Bucket>>> {
    let result = S3_CLIENT.list_buckets().send().await?;
    Ok(result.buckets)
}

static S3_CLIENT: Lazy<s3::Client> = Lazy::new(|| {
    let config = futures::executor::block_on(
        init_config_loader(get_var::<String>("S3_ENDPOINT_URL").ok()).load(),
    );
    s3::Client::new(&config)
});
