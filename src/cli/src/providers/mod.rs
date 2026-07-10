use async_trait::async_trait;

/// 网盘存储提供商公共接口
#[async_trait]
pub trait StorageProvider: Send + Sync {
    /// 提供商名称标识
    fn name(&self) -> &'static str;

    /// 发送文件：上传并生成分享链接，返回可分享的 URL
    async fn send(&self, local_path: &str, remote_path: &str) -> Result<String, String>;

    /// 接收文件：从分享链接下载到本地
    async fn receive(&self, url: &str, local_path: &str) -> Result<(), String>;
}

pub mod baidu_drive;
pub mod dropbox;
pub mod google_drive;
pub mod onedrive;
pub mod quark_drive;

pub use baidu_drive::BaiduDriveProvider;
pub use dropbox::DropboxProvider;
pub use google_drive::GoogleDriveProvider;
pub use onedrive::OneDriveProvider;
pub use quark_drive::QuarkDriveProvider;

/// 根据名称创建提供商实例
pub fn from_name(name: &str) -> Option<Box<dyn StorageProvider>> {
    match name {
        "dropbox" => Some(Box::new(DropboxProvider)),
        "baidu" | "baidudrive" => Some(Box::new(BaiduDriveProvider)),
        "google" | "googledrive" => Some(Box::new(GoogleDriveProvider)),
        "onedrive" => Some(Box::new(OneDriveProvider)),
        "quark" | "quarkdrive" => Some(Box::new(QuarkDriveProvider)),
        _ => None,
    }
}

/// 从分享链接 URL 自动识别提供商
pub fn detect(url: &str) -> Option<Box<dyn StorageProvider>> {
    if url.contains("dropbox.com") {
        Some(Box::new(DropboxProvider))
    } else if url.contains("pan.baidu.com") {
        Some(Box::new(BaiduDriveProvider))
    } else if url.contains("drive.google.com") {
        Some(Box::new(GoogleDriveProvider))
    } else if url.contains("1drv.ms") || url.contains("onedrive.live.com") {
        Some(Box::new(OneDriveProvider))
    } else if url.contains("pan.quark.cn") {
        Some(Box::new(QuarkDriveProvider))
    } else {
        None
    }
}
