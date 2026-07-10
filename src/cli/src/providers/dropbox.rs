use async_trait::async_trait;
use reqwest::Client;
use std::fs;

use super::StorageProvider;

pub struct DropboxProvider;

/// Dropbox 内部 upload 实现（带 mock 支持，供测试用）
pub async fn upload(token: &str, local_path: &str, remote_path: &str, mock_base: Option<&str>) {
    let data = fs::read(local_path).expect("读取本地文件失败");
    let client = Client::new();

    let base = mock_base
        .map(|s| s.to_string())
        .unwrap_or_else(|| "https://content.dropboxapi.com".to_string());

    let arg = serde_json::json!({
        "path": remote_path,
        "mode": "overwrite",
    });

    let resp = client
        .post(format!("{base}/files/upload"))
        .header("Authorization", format!("Bearer {token}"))
        .header("Dropbox-API-Arg", arg.to_string())
        .header("Content-Type", "application/octet-stream")
        .body(data.clone())
        .send()
        .await
        .expect("上传请求失败");

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        panic!("上传失败 [{status}]: {text}");
    }

    println!(
        "✓ 已上传: {local_path} → {remote_path} ({} 字节)",
        data.len()
    );
}

/// Dropbox 内部 create_shared_link 实现（带 mock 支持，供测试用）
pub async fn create_shared_link(
    token: &str,
    path: &str,
    mock_base: Option<&str>,
) -> Result<String, String> {
    let client = Client::new();
    let base = mock_base
        .map(|s| s.to_string())
        .unwrap_or_else(|| "https://api.dropboxapi.com".to_string());

    let body = serde_json::json!({
        "path": path,
        "settings": { "requested_visibility": { ".tag": "public" } }
    });

    let resp = client
        .post(format!("{base}/sharing/create_shared_link_with_settings"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("请求失败: {e}"))?;

    let json: serde_json::Value = resp.json().await.map_err(|e| format!("解析失败: {e}"))?;

    if let Some(url) = json["url"].as_str() {
        Ok(url.replace("?dl=0", "?dl=1"))
    } else if let Some(error) = json["error_summary"].as_str() {
        Err(error.to_string())
    } else {
        Err(format!("未知响应: {json}"))
    }
}

#[async_trait]
impl StorageProvider for DropboxProvider {
    fn name(&self) -> &'static str {
        "dropbox"
    }

    async fn send(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let token = std::env::var("DROPBOX_ACCESS_TOKEN")
            .map_err(|_| "请设置 DROPBOX_ACCESS_TOKEN".to_string())?;
        // 上传
        upload(&token, local_path, remote_path, None).await;
        // 生成分享链接
        create_shared_link(&token, remote_path, None).await
    }

    async fn receive(&self, url: &str, local_path: &str) -> Result<(), String> {
        let client = Client::new();
        let dl_url = if url.contains("?dl=") {
            url.to_string()
        } else {
            format!("{url}?dl=1")
        };

        let resp = client
            .get(&dl_url)
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
