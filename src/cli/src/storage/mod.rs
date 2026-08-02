use async_trait::async_trait;

/// 网盘存储提供商公共接口
#[async_trait]
pub trait Storage: Send + Sync {
    /// 提供商名称标识
    fn name(&self) -> &'static str;

    /// 发送文件：上传并生成分享链接，返回可分享的 URL
    async fn send(&self, local_path: &str, remote_path: &str) -> Result<String, String>;

    /// 接收文件：从分享链接下载到本地
    async fn receive(&self, url: &str, local_path: &str) -> Result<(), String>;

    /// 自动接收：直接从远程路径拉取文件（需直接访问权限）
    /// S3、SFTP 等支持，网盘类不支持
    async fn receive_path(&self, _remote: &str, _local: &str) -> Result<(), String> {
        Err("该平台不支持自动接收，请提供分享链接".to_string())
    }
}

pub mod baidu_drive;
pub mod dropbox;
pub mod google_drive;
pub mod onedrive;
pub mod s3;
pub mod sftp;

pub use baidu_drive::BaiduDriveStorage;
pub use dropbox::DropboxStorage;
pub use google_drive::GoogleDriveStorage;
pub use onedrive::OneDriveStorage;
pub use s3::S3Storage;
pub use sftp::SftpStorage;

// ── 兼容层（v0.2.x 旧名，随 v0.3 移除）──
#[deprecated(note = "更名为 BaiduDriveStorage")]
pub use BaiduDriveStorage as BaiduDriveProvider;
#[deprecated(note = "更名为 DropboxStorage")]
pub use DropboxStorage as DropboxProvider;
#[deprecated(note = "更名为 GoogleDriveStorage")]
pub use GoogleDriveStorage as GoogleDriveProvider;
#[deprecated(note = "更名为 OneDriveStorage")]
pub use OneDriveStorage as OneDriveProvider;
#[deprecated(note = "更名为 S3Storage")]
pub use S3Storage as S3Provider;
#[deprecated(note = "更名为 SftpStorage")]
pub use SftpStorage as SftpProvider;
#[deprecated(note = "更名为 Storage")]
pub use Storage as StorageProvider;

/// 根据名称创建提供商实例
pub fn from_name(name: &str) -> Option<Box<dyn Storage>> {
    match name {
        "dropbox" => Some(Box::new(DropboxStorage)),
        "baidu" | "baidudrive" => Some(Box::new(BaiduDriveStorage)),
        "google" | "googledrive" => Some(Box::new(GoogleDriveStorage)),
        "onedrive" => Some(Box::new(OneDriveStorage)),
        "s3" => Some(Box::new(S3Storage)),
        "sftp" => Some(Box::new(SftpStorage)),
        _ => None,
    }
}

/// 从分享链接 URL 自动识别提供商
pub fn detect(url: &str) -> Option<Box<dyn Storage>> {
    if url.contains("dropbox.com") {
        Some(Box::new(DropboxStorage))
    } else if url.contains("pan.baidu.com") {
        Some(Box::new(BaiduDriveStorage))
    } else if url.contains("drive.google.com") {
        Some(Box::new(GoogleDriveStorage))
    } else if url.contains("1drv.ms") || url.contains("onedrive.live.com") {
        Some(Box::new(OneDriveStorage))
    } else if url.contains("s3.amazonaws.com") || url.contains("s3.") {
        Some(Box::new(S3Storage))
    } else if url.starts_with("sftp://") {
        Some(Box::new(SftpStorage))
    } else {
        None
    }
}
