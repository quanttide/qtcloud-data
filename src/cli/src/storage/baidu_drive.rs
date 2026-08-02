use async_trait::async_trait;
use reqwest::Client;
use std::fs;

use super::StorageProvider;

pub struct BaiduDriveProvider;

const API_BASE: &str = "https://pan.baidu.com/rest/2.0/xpan/file";
const SHARE_API: &str = "https://pan.baidu.com/rest/2.0/xpan/share";

impl BaiduDriveProvider {
    fn token(&self) -> Result<String, String> {
        std::env::var("BAIDU_ACCESS_TOKEN")
            .or_else(|_| std::env::var("BAIDUDRIVE_ACCESS_TOKEN"))
            .map_err(|_| "请设置 BAIDU_ACCESS_TOKEN 环境变量".to_string())
    }

    fn client(&self) -> Client {
        Client::new()
    }
}

#[async_trait]
impl StorageProvider for BaiduDriveProvider {
    fn name(&self) -> &'static str {
        "baidudrive"
    }

    async fn send(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let token = self.token()?;
        let data = fs::read(local_path).map_err(|e| format!("读取文件失败: {e}"))?;
        let size = data.len();
        let client = self.client();
        let _file_name = local_path.rsplit('/').next().unwrap_or("file");

        // 1. 预创建文件
        let precreate_url = format!("{API_BASE}?method=precreate&access_token={token}");
        let precreate_body = serde_json::json!({
            "path": remote_path,
            "size": size,
            "isdir": 0,
            "block_list": ["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"], // 占位符，autoinit 后服务端覆盖
            "autoinit": 1,
        });

        let pre_resp = client
            .post(&precreate_url)
            .json(&precreate_body)
            .send()
            .await
            .map_err(|e| format!("预创建请求失败: {e}"))?;

        let pre_json: serde_json::Value = pre_resp
            .json()
            .await
            .map_err(|e| format!("解析预创建响应失败: {e}"))?;

        let uploadid = pre_json["uploadid"]
            .as_str()
            .ok_or_else(|| format!("预创建失败: {pre_json}"))?;

        // 2. 上传文件内容
        let upload_url = format!(
            "{API_BASE}?method=upload&access_token={token}&type=tmpfile&path={remote_path}&uploadid={uploadid}&partseq=0"
        );

        client
            .post(&upload_url)
            .body(data)
            .send()
            .await
            .map_err(|e| format!("上传请求失败: {e}"))?;

        // 3. 创建文件
        let create_url = format!("{API_BASE}?method=create&access_token={token}");
        let create_body = serde_json::json!({
            "path": remote_path,
            "size": size,
            "isdir": 0,
            "uploadid": uploadid,
            "block_list": ["xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"],
        });

        let create_resp = client
            .post(&create_url)
            .json(&create_body)
            .send()
            .await
            .map_err(|e| format!("创建文件请求失败: {e}"))?;

        let create_json: serde_json::Value = create_resp
            .json()
            .await
            .map_err(|e| format!("解析创建响应失败: {e}"))?;

        let _fsid = create_json["fs_id"]
            .as_i64()
            .ok_or_else(|| format!("创建文件失败: {create_json}"))?;

        // 4. 创建分享链接
        let share_url = format!("{SHARE_API}?method=create&access_token={token}");
        let share_body = serde_json::json!({
            "path": [remote_path],
            "period": 7,          // 7天有效期
            "share_type": 2,      // 公开链接
        });

        let share_resp = client
            .post(&share_url)
            .json(&share_body)
            .send()
            .await
            .map_err(|e| format!("创建分享请求失败: {e}"))?;

        let share_json: serde_json::Value = share_resp
            .json()
            .await
            .map_err(|e| format!("解析分享响应失败: {e}"))?;

        let link = share_json["link"]
            .as_str()
            .ok_or_else(|| format!("创建分享失败: {share_json}"))?;

        println!("✓ 已上传: {local_path} → {remote_path} ({} 字节)", size);

        // 如有提取码一并返回
        let pwd = share_json["pwd"].as_str().unwrap_or("");
        if pwd.is_empty() {
            Ok(link.to_string())
        } else {
            Ok(format!("{link}?pwd={pwd}"))
        }
    }

    async fn receive(&self, url: &str, local_path: &str) -> Result<(), String> {
        let client = self.client();
        let token = self.token()?;

        // 从分享链接提取 surl
        let surl = url
            .split("surl=")
            .nth(1)
            .or_else(|| url.split("/s/").nth(1)?.split('?').next())
            .ok_or_else(|| format!("无法从链接提取 surl: {url}"))?;
        let surl = surl.trim_end_matches('/');

        // 解析分享信息：获取文件列表
        let info_url = format!("{SHARE_API}?method=list&access_token={token}");

        let info_body = serde_json::json!({
            "shorturl": surl,
        });

        let info_resp = client
            .post(&info_url)
            .json(&info_body)
            .send()
            .await
            .map_err(|e| format!("查询分享信息失败: {e}"))?;

        let info_json: serde_json::Value = info_resp
            .json()
            .await
            .map_err(|e| format!("解析分享信息失败: {e}"))?;

        let list = info_json["list"]
            .as_array()
            .ok_or_else(|| format!("未找到分享文件: {info_json}"))?;

        let first = list.first().ok_or_else(|| "分享文件列表为空".to_string())?;

        let _fs_id = first["fs_id"]
            .as_i64()
            .ok_or_else(|| format!("无法获取 fs_id: {first}"))?;

        let dlink = first["dlink"]
            .as_str()
            .ok_or_else(|| format!("无法获取下载链接: {first}"))?;

        // 拼接带 token 的下载链接
        let dl_url = format!("{dlink}&access_token={token}");

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
