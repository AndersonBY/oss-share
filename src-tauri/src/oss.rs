use serde::{Deserialize, Serialize};
use std::path::Path;

use ali_oss_rs::bucket::BucketOperations;
use ali_oss_rs::bucket_common::ListObjectsOptionsBuilder;
use ali_oss_rs::object::ObjectOperations;
use ali_oss_rs::presign_common::PresignGetOptions;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OssFileInfo {
    pub key: String,
    pub size: u64,
    pub last_modified: String,
}

pub struct OssService {
    client: ali_oss_rs::Client,
    bucket: String,
    prefix: String,
}

impl OssService {
    /// Create a new OssService.
    ///
    /// `region` should be an OSS region identifier like "oss-cn-hangzhou".
    /// The endpoint is derived as "{region}.aliyuncs.com" and the SDK region
    /// is extracted by stripping the "oss-" prefix.
    pub fn new(
        access_key_id: &str,
        access_key_secret: &str,
        region: &str,
        bucket: &str,
        prefix: &str,
    ) -> Result<Self, String> {
        if access_key_id.is_empty() || access_key_secret.is_empty() {
            return Err("Access key ID and secret must not be empty".into());
        }
        if bucket.is_empty() {
            return Err("Bucket name must not be empty".into());
        }

        // region is like "oss-cn-hangzhou", endpoint is "oss-cn-hangzhou.aliyuncs.com"
        // ali-oss-rs expects region without "oss-" prefix, e.g. "cn-hangzhou"
        let endpoint = format!("{}.aliyuncs.com", region);
        let sdk_region = region.strip_prefix("oss-").unwrap_or(region);

        let client = ali_oss_rs::ClientBuilder::new(access_key_id, access_key_secret, &endpoint)
            .region(sdk_region)
            .build()
            .map_err(|e| format!("Failed to create OSS client: {}", e))?;

        Ok(Self {
            client,
            bucket: bucket.to_string(),
            prefix: prefix.to_string(),
        })
    }

    /// Upload a file to OSS. Returns the object key.
    pub async fn upload_file(&self, file_path: &Path) -> Result<String, String> {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| "Invalid file name".to_string())?;

        let object_key = format!("{}{}", self.prefix, file_name);

        self.client
            .put_object_from_file(&self.bucket, &object_key, file_path, None)
            .await
            .map_err(|e| format!("Upload failed: {}", e))?;

        Ok(object_key)
    }

    /// Generate a presigned URL for downloading an object.
    pub fn generate_presigned_url(
        &self,
        object_key: &str,
        expire_seconds: u64,
    ) -> Result<String, String> {
        let options = PresignGetOptions {
            expire_seconds: expire_seconds as u32,
            ..Default::default()
        };

        let url = self.client.presign_url(&self.bucket, object_key, options);
        Ok(url)
    }

    /// List files under the configured prefix.
    pub async fn list_files(&self) -> Result<Vec<OssFileInfo>, String> {
        let options = ListObjectsOptionsBuilder::new()
            .prefix(&self.prefix)
            .max_keys(1000)
            .build();

        let result = self
            .client
            .list_objects(&self.bucket, Some(options))
            .await
            .map_err(|e| format!("List objects failed: {}", e))?;

        let files = result
            .contents
            .into_iter()
            .map(|obj| OssFileInfo {
                key: obj.key,
                size: obj.size,
                last_modified: obj.last_modified,
            })
            .collect();

        Ok(files)
    }

    /// Delete an object from OSS.
    pub async fn delete_file(&self, object_key: &str) -> Result<(), String> {
        self.client
            .delete_object(&self.bucket, object_key, None)
            .await
            .map_err(|e| format!("Delete failed: {}", e))?;

        Ok(())
    }

    /// Test the connection by listing objects (max 1) under the prefix.
    pub async fn test_connection(&self) -> Result<(), String> {
        let options = ListObjectsOptionsBuilder::new()
            .prefix(&self.prefix)
            .max_keys(1)
            .build();

        self.client
            .list_objects(&self.bucket, Some(options))
            .await
            .map_err(|e| format!("Connection test failed: {}", e))?;

        Ok(())
    }
}
