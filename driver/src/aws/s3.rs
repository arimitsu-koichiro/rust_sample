use crate::aws::with_profile;
use anyhow::Result;
use rusoto_core::{HttpClient, Region};
use rusoto_s3::{Bucket, S3Client, S3};

fn get_client() -> S3Client {
    with_profile(|x| match x {
        Some(provider) => {
            S3Client::new_with(HttpClient::new().unwrap(), provider, Region::default())
        }
        _ => S3Client::new(Region::default()),
    })
}

#[allow(dead_code)]
pub async fn ls() -> Result<Option<Vec<Bucket>>> {
    let client = get_client();
    let result = client.list_buckets().await?;
    Ok(result.buckets)
}
