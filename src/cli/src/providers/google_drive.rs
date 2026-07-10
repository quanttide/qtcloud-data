use async_trait::async_trait;
use reqwest::Client;
use std::fs;

use super::StorageProvider;

pub struct GoogleDriveProvider;

const API_BASE: &str = "https://www.googleapis.com/drive/v3";
const UPLOAD_BASE: &str = "https://www.googleapis.com/upload/drive/v3";

impl GoogleDriveProvider {
    fn token(&self) -> Result<String, String> {
        std::env::var("GOOGLE_DRIVE_ACCESS_TOKEN")
            .or_else(|_| std::env::var("GDRIVE_ACCESS_TOKEN"))
            .map_err(|_| "请设置 GOOGLE_DRIVE_ACCESS_TOKEN 环境变量".to_string())
    }

    fn client(&self) -> Client {
        Client::new()
    }
}

#[async_trait]
impl StorageProvider for GoogleDriveProvider {
    fn name(&self) -> &'static str {
        "googledrive"
    }

    async fn send(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let token = self.token()?;
        let data = fs::read(local_path).map_err(|e| format!("读取文件失败: {e}"))?;
        let client = self.client();
        let file_name = remote_path.rsplit('/').next().unwrap_or("file");
        let mime = mime_guess::from_path(local_path)
            .first_or_octet_stream()
            .to_string();

        // 1. 先创建文件元数据（空 body），获取 fileId
        let metadata = serde_json::json!({
            "name": file_name,
            "parents": parent_folder_id(remote_path),
        });

        let create_resp = client
            .post(format!("{UPLOAD_BASE}/files?uploadType=resumable"))
            .header("Authorization", format!("Bearer {token}"))
            .header("Content-Type", "application/json; charset=UTF-8")
            .header("X-Upload-Content-Type", &mime)
            .header("X-Upload-Content-Length", data.len().to_string())
            .json(&metadata)
            .send()
            .await
            .map_err(|e| format!("创建文件请求失败: {e}"))?;

        // 从 Location header 获取上传 URL
        let upload_url = create_resp
            .headers()
            .get("location")
            .ok_or_else(|| "未获取到上传 URL".to_string())?
            .to_str()
            .map_err(|e| format!("解析上传 URL 失败: {e}"))?
            .to_string();

        // 2. 上传文件内容
        client
            .put(&upload_url)
            .header("Content-Type", &mime)
            .body(data.clone())
            .send()
            .await
            .map_err(|e| format!("上传请求失败: {e}"))?;

        // 3. 获取 fileId（从上传 URL 中提取）
        let file_id = upload_url
            .split("files/")
            .nth(1)
            .and_then(|s| s.split('?').next())
            .ok_or_else(|| "无法提取 fileId".to_string())?;

        // 4. 设置权限（任何人可读）
        let permission = serde_json::json!({
            "type": "anyone",
            "role": "reader",
        });

        client
            .post(format!("{API_BASE}/files/{file_id}/permissions"))
            .header("Authorization", format!("Bearer {token}"))
            .json(&permission)
            .send()
            .await
            .map_err(|e| format!("设置权限失败: {e}"))?;

        // 5. 获取分享链接
        let info_resp = client
            .get(format!("{API_BASE}/files/{file_id}?fields=webViewLink"))
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| format!("获取文件信息失败: {e}"))?;

        let info_json: serde_json::Value = info_resp
            .json()
            .await
            .map_err(|e| format!("解析文件信息失败: {e}"))?;

        let link = info_json["webViewLink"]
            .as_str()
            .ok_or_else(|| format!("未获取到 webViewLink: {info_json}"))?;

        println!(
            "✓ 已上传: {local_path} → {remote_path} ({} 字节)",
            data.len()
        );

        Ok(link.to_string())
    }

    async fn receive(&self, url: &str, local_path: &str) -> Result<(), String> {
        let token = self.token()?;
        let client = self.client();

        // 从 URL 提取 fileId
        // Google Drive 分享链接格式: https://drive.google.com/file/d/{fileId}/view
        let file_id = url
            .split("/file/d/")
            .nth(1)
            .and_then(|s| s.split('/').next())
            .ok_or_else(|| format!("无法从 URL 提取 fileId: {url}"))?;

        // 下载文件
        let resp = client
            .get(format!("{API_BASE}/files/{file_id}?alt=media"))
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

/// 从远程路径中提取父文件夹 ID
/// 格式：/folderId/filename → "folderId"
/// 如果是根目录则返回空数组
fn parent_folder_id(remote_path: &str) -> Vec<String> {
    let parts: Vec<&str> = remote_path.trim_start_matches('/').split('/').collect();
    if parts.len() > 1 {
        // 第一个 segment 作为文件夹 ID
        vec![parts[0..parts.len() - 1].join("/")]
    } else {
        // 根目录
        vec![]
    }
}
