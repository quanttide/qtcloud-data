use async_trait::async_trait;
use reqwest::Client;
use std::fs;

use super::StorageProvider;

pub struct OneDriveProvider;

const GRAPH_API: &str = "https://graph.microsoft.com/v1.0";

impl OneDriveProvider {
    fn token(&self) -> Result<String, String> {
        std::env::var("ONEDRIVE_ACCESS_TOKEN")
            .or_else(|_| std::env::var("OD_ACCESS_TOKEN"))
            .map_err(|_| "请设置 ONEDRIVE_ACCESS_TOKEN 环境变量".to_string())
    }

    fn client(&self) -> Client {
        Client::new()
    }
}

#[async_trait]
impl StorageProvider for OneDriveProvider {
    fn name(&self) -> &'static str {
        "onedrive"
    }

    async fn send(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let token = self.token()?;
        let data = fs::read(local_path).map_err(|e| format!("读取文件失败: {e}"))?;
        let client = self.client();
        let size = data.len();

        // 1. 简单上传（适用于 < 4MB 文件，大文件需分片）
        // path 格式: /drive/root:/path/to/file:/content
        let upload_url = format!(
            "{GRAPH_API}/me/drive/root:/{}:/content",
            remote_path.trim_start_matches('/')
        );

        client
            .put(&upload_url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/octet-stream")
            .body(data)
            .send()
            .await
            .map_err(|e| format!("上传请求失败: {e}"))?;

        // 2. 创建分享链接（仅查看）
        let share_url = format!(
            "{GRAPH_API}/me/drive/root:/{}:/createLink",
            remote_path.trim_start_matches('/')
        );
        let share_body = serde_json::json!({
            "type": "view",
            "scope": "anonymous",
        });

        let share_resp = client
            .post(&share_url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&share_body)
            .send()
            .await
            .map_err(|e| format!("创建分享链接失败: {e}"))?;

        let share_json: serde_json::Value = share_resp
            .json()
            .await
            .map_err(|e| format!("解析分享响应失败: {e}"))?;

        let link = share_json["link"]["webUrl"]
            .as_str()
            .ok_or_else(|| format!("创建分享失败: {share_json}"))?
            .to_string();

        println!("✓ 已上传: {local_path} → {remote_path} ({size} 字节)");
        Ok(link)
    }

    async fn receive(&self, url: &str, local_path: &str) -> Result<(), String> {
        let token = self.token()?;
        let client = self.client();

        // 从分享链接提取共享标识
        // OneDrive 链接格式: https://1drv.ms/u/s!... 或 https://onedrive.live.com/...
        // 简化：直接尝试用 /sharedWithMe API 或从 URL 获取文件
        // 实际上 OneDrive 的共享链接消费需要先 resolve 为可下载的 CDN URL

        // 尝试直接下载：在 URL 后加 ?download=1
        let dl_url = if url.contains('?') {
            format!("{url}&download=1")
        } else {
            format!("{url}?download=1")
        };

        // 部分链接需要 redirect 跟随到真实 CDN
        let resp = client
            .get(&dl_url)
            .header("Authorization", format!("Bearer {token}"))
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
        fs::write(local_path, &bytes).map_err(|e| format!("写入文件失败: {e}"))?;
        println!("✓ 已接收: {local_path} ({} 字节)", bytes.len());
        Ok(())
    }
}
