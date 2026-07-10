use async_trait::async_trait;
use reqwest::Client;

use super::StorageProvider;

pub struct QuarkDriveProvider;

/// 夸克网盘 API 端点（社区反向工程，可能随版本变化）
const QUARK_API: &str = "https://drive.quark.cn/1";

impl QuarkDriveProvider {
    /// 夸克使用 Cookie + token 认证，非标准 OAuth
    /// 需从浏览器开发者工具中获取
    fn cookie(&self) -> Result<String, String> {
        std::env::var("QUARK_COOKIE")
            .map_err(|_| "请设置 QUARK_COOKIE 环境变量（从浏览器开发者工具复制）".to_string())
    }

    fn client(&self) -> Client {
        Client::builder()
            .cookie_store(true)
            .build()
            .expect("创建 HTTP 客户端失败")
    }
}

use std::fs;

#[async_trait]
impl StorageProvider for QuarkDriveProvider {
    fn name(&self) -> &'static str {
        "quark"
    }

    async fn send(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let cookie = self.cookie()?;
        let data = fs::read(local_path).map_err(|e| format!("读取文件失败: {e}"))?;
        let client = self.client();
        let file_name = local_path.rsplit('/').next().unwrap_or("file");
        let size = data.len();

        // 1. 获取上传凭证（pre-create）
        let pre_url = format!("{QUARK_API}/file/upload/precreate");
        let pre_body = serde_json::json!({
            "file_name": file_name,
            "file_size": size,
            "dir_path": remote_path,
        });

        let pre_resp = client
            .post(&pre_url)
            .header("Cookie", &cookie)
            .json(&pre_body)
            .send()
            .await
            .map_err(|e| format!("预创建请求失败: {e}"))?;

        let pre_json: serde_json::Value = pre_resp
            .json()
            .await
            .map_err(|e| format!("解析预创建响应失败: {e}"))?;

        let upload_token = pre_json["data"]["upload_token"]
            .as_str()
            .ok_or_else(|| format!("预创建失败: {pre_json}"))?
            .to_string();

        let file_id = pre_json["data"]["file_id"]
            .as_str()
            .ok_or_else(|| format!("预创建失败: {pre_json}"))?
            .to_string();

        // 2. 上传文件内容
        let upload_url = format!("{QUARK_API}/file/upload");
        // 夸克上传使用 multipart/form-data
        // 构建上传请求
        let upload_resp = client
            .post(&upload_url)
            .header("Cookie", &cookie)
            .multipart(
                reqwest::multipart::Form::new()
                    .text("upload_token", upload_token.clone())
                    .text("file_id", file_id.clone())
                    .text("part_number", "1")
                    .part(
                        "file",
                        reqwest::multipart::Part::bytes(data.clone())
                            .file_name(file_name.to_string()),
                    ),
            )
            .send()
            .await
            .map_err(|e| format!("上传请求失败: {e}"))?;

        if !upload_resp.status().is_success() {
            let status = upload_resp.status();
            let text = upload_resp.text().await.unwrap_or_default();
            return Err(format!("上传失败 [{status}]: {text}"));
        }

        // 3. 完成上传（commit）
        let commit_url = format!("{QUARK_API}/file/upload/complete");
        let commit_body = serde_json::json!({
            "upload_token": upload_token,
            "file_id": file_id,
        });

        let _commit_resp = client
            .post(&commit_url)
            .header("Cookie", &cookie)
            .json(&commit_body)
            .send()
            .await
            .map_err(|e| format!("完成上传请求失败: {e}"))?;

        // 4. 创建分享
        let share_url = format!("{QUARK_API}/share/create");
        let share_body = serde_json::json!({
            "file_ids": [file_id],
            "share_type": "public",
            "expire_type": 1, // 7天有效期
        });

        let share_resp = client
            .post(&share_url)
            .header("Cookie", &cookie)
            .json(&share_body)
            .send()
            .await
            .map_err(|e| format!("创建分享失败: {e}"))?;

        let share_json: serde_json::Value = share_resp
            .json()
            .await
            .map_err(|e| format!("解析分享响应失败: {e}"))?;

        let share_link = share_json["data"]["share_url"]
            .as_str()
            .ok_or_else(|| format!("创建分享失败: {share_json}"))?
            .to_string();

        let share_pwd = share_json["data"]["share_pwd"].as_str().unwrap_or("");

        println!("✓ 已上传: {local_path} → {remote_path} ({size} 字节)");

        if share_pwd.is_empty() {
            Ok(share_link)
        } else {
            Ok(format!("{share_link}?pwd={share_pwd}"))
        }
    }

    async fn receive(&self, url: &str, local_path: &str) -> Result<(), String> {
        let cookie = self.cookie()?;
        let client = self.client();

        // 夸克分享链接格式: https://pan.quark.cn/s/{share_id}
        let share_id = url
            .split("/s/")
            .nth(1)
            .or_else(|| url.split("share_id=").nth(1)?.split('&').next())
            .ok_or_else(|| format!("无法从链接提取 share_id: {url}"))?;
        let share_id = share_id.trim_end_matches('/');

        // 1. 获取分享信息
        let info_url = format!("{QUARK_API}/share/{share_id}/detail");

        let info_resp = client
            .get(&info_url)
            .header("Cookie", &cookie)
            .send()
            .await
            .map_err(|e| format!("查询分享信息失败: {e}"))?;

        let info_json: serde_json::Value = info_resp
            .json()
            .await
            .map_err(|e| format!("解析分享信息失败: {e}"))?;

        let list = info_json["data"]["file_list"]
            .as_array()
            .ok_or_else(|| format!("未找到分享文件: {info_json}"))?;

        let first = list.first().ok_or_else(|| "分享文件列表为空".to_string())?;

        let file_id = first["file_id"]
            .as_str()
            .ok_or_else(|| format!("无法获取 file_id: {first}"))?;

        let file_name = first["file_name"].as_str().unwrap_or("downloaded");

        // 2. 获取下载链接
        let dl_url = format!("{QUARK_API}/file/download?file_id={file_id}");

        let dl_resp = client
            .get(&dl_url)
            .header("Cookie", &cookie)
            .send()
            .await
            .map_err(|e| format!("获取下载链接失败: {e}"))?;

        let dl_json: serde_json::Value = dl_resp
            .json()
            .await
            .map_err(|e| format!("解析下载响应失败: {e}"))?;

        let real_url = dl_json["data"]["download_url"]
            .as_str()
            .ok_or_else(|| format!("未获取到下载链接: {dl_json}"))?;

        // 3. 下载文件
        let resp = client
            .get(real_url)
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

        // 保存路径：有指定就用指定，否则用文件名
        let save_path = if local_path.ends_with('/') || local_path.ends_with("\\") {
            format!("{local_path}{file_name}")
        } else {
            local_path.to_string()
        };

        fs::write(&save_path, &bytes).map_err(|e| format!("写入文件失败: {e}"))?;
        println!("✓ 已接收: {save_path} ({} 字节)", bytes.len());
        Ok(())
    }
}
