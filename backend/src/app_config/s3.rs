use clap::error::ErrorKind;
use serde::{Deserialize, Serialize};

use crate::{
    app_config::ConfigItem,
    error::fatal::{FatalError, FatalResult, MultiFatalError},
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct StaticS3Config {
    /// AWS region (e.g., "us-east-1")
    pub region: String,

    /// S3 bucket name
    pub bucket: String,

    /// AWS access key ID (optional, uses AWS credentials chain if not provided)
    pub access_key_id: Option<String>,

    /// AWS secret access key (optional, uses AWS credentials chain if not provided)
    pub secret_access_key: Option<String>,

    /// Custom S3 endpoint (for MinIO or other S3-compatible services)
    pub endpoint: Option<String>,

    /// Use path-style addressing (required for some S3-compatible services)
    pub force_path_style: bool,

    /// CDN domain for public asset URLs (optional)
    pub cdn_domain: Option<String>,

    /// Key prefix for organizing objects (e.g., "assets/")
    pub key_prefix: Option<String>,

    /// TTL for presigned url, default 900s
    pub url_ttl: u64,
}

pub type S3Config = StaticS3Config;

impl Default for StaticS3Config {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            bucket: String::new(),
            access_key_id: None,
            secret_access_key: None,
            endpoint: None,
            force_path_style: true,
            cdn_domain: None,
            key_prefix: None,
            url_ttl: 900
        }
    }
}

impl ConfigItem for StaticS3Config {
    type RuntimeConfig = S3Config;

    fn into_runtime(self) -> FatalResult<Self::RuntimeConfig> {
        if self.bucket.is_empty() {
            let mut errors = MultiFatalError::new();
            errors.push(FatalError::new(
                ErrorKind::Io,
                "S3 bucket must be configured".to_string(),
                None,
            ));
            return Err(errors);
        }

        Ok(self)
    }
}

impl S3Config {
    // # 获取公开访问的 URL（使用 CDN、自定义端点或 AWS S3）
    // pub fn public_url(&self, key: &str) -> String {
    //     let full_key = self.full_key(key);
    //
    //     // 1. CDN 优先
    //     if let Some(cdn) = &self.cdn_domain {
    //         // CDN URL（强制 HTTPS）
    //         return format!(
    //             "https://{}/{}",
    //             cdn.trim_start_matches("https://")
    //                 .trim_start_matches("http://")
    //                 .trim_end_matches('/'),
    //             full_key
    //         );
    //     }
    //
    //     // 2. 自定义端点（MinIO、LocalStack 等）
    //     if let Some(endpoint) = &self.endpoint {
    //         let endpoint_str = endpoint.as_str();
    //         // 确定协议和主机
    //         let (protocol, host) = if endpoint_str.starts_with("https://") {
    //             ("https://", endpoint_str.trim_start_matches("https://"))
    //         } else if endpoint_str.starts_with("http://") {
    //             ("http://", endpoint_str.trim_start_matches("http://"))
    //         } else {
    //             // 默认 HTTPS
    //             ("https://", endpoint_str)
    //         };
    //         let host = host.trim_end_matches('/');
    //         // 使用路径样式：{protocol}{host}/{bucket}/{key}
    //         return format!("{}{}/{}/{}", protocol, host, self.bucket, full_key);
    //     }
    //
    //     // 3. AWS S3（回退）
    //     if self.force_path_style {
    //         // Path-style URL
    //         format!(
    //             "https://s3.{}.amazonaws.com/{}/{}",
    //             self.region, self.bucket, full_key
    //         )
    //     } else {
    //         // Virtual-hosted style URL
    //         format!(
    //             "https://{}.s3.{}.amazonaws.com/{}",
    //             self.bucket, self.region, full_key
    //         )
    //     }
    // }
}
