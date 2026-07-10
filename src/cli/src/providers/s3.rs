use async_trait::async_trait;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use reqwest::Client;

use super::StorageProvider;

pub struct S3Provider;

impl S3Provider {
    async fn client(&self) -> Result<aws_sdk_s3::Client, String> {
        let config = aws_config::load_from_env().await;
        Ok(aws_sdk_s3::Client::new(&config))
    }

    fn bucket(&self) -> String {
        std::env::var("S3_BUCKET").unwrap_or_else(|_| "qtcloud-transfer".to_string())
    }
}

#[async_trait]
impl StorageProvider for S3Provider {
    fn name(&self) -> &'static str {
        "s3"
    }

    async fn send(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let client = self.client().await?;
        let bucket = self.bucket();
        let key = remote_path.trim_start_matches('/');
        let data = tokio::fs::read(local_path)
            .await
            .map_err(|e| format!("读取文件失败: {e}"))?;
        let size = data.len();

        // 上传到 S3
        client
            .put_object()
            .bucket(&bucket)
            .key(key)
            .body(ByteStream::from(data))
            .send()
            .await
            .map_err(|e| format!("上传失败: {e}"))?;

        // 生成预签名 GET URL（7 天有效期）
        let expires = PresigningConfig::builder()
            .expires_in(std::time::Duration::from_secs(7 * 24 * 3600))
            .build()
            .map_err(|e| format!("配置签名失败: {e}"))?;

        let presigned = client
            .get_object()
            .bucket(&bucket)
            .key(key)
            .presigned(expires)
            .await
            .map_err(|e| format!("生成预签名 URL 失败: {e}"))?;

        println!("✓ 已上传: {local_path} → s3://{bucket}/{key} ({size} 字节)");
        Ok(presigned.uri().to_string())
    }

    async fn receive(&self, url: &str, local_path: &str) -> Result<(), String> {
        let client = Client::new();

        let resp = client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("下载请求失败: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("下载失败 [{status}]: {text}"));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("读取响应失败: {e}"))?;
        tokio::fs::write(local_path, &bytes)
            .await
            .map_err(|e| format!("写入文件失败: {e}"))?;
        println!("✓ 已接收: {local_path} ({} 字节)", bytes.len());
        Ok(())
    }
}
