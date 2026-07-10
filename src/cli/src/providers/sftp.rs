use async_trait::async_trait;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

use super::StorageProvider;

pub struct SftpProvider;

impl SftpProvider {
    fn connect_from_env() -> Result<(ssh2::Session, ssh2::Sftp), String> {
        let host =
            std::env::var("SFTP_HOST").map_err(|_| "请设置 SFTP_HOST 环境变量".to_string())?;
        let port = std::env::var("SFTP_PORT")
            .unwrap_or_else(|_| "22".to_string())
            .parse::<u16>()
            .map_err(|_| "SFTP_PORT 格式错误".to_string())?;
        let user = std::env::var("SFTP_USER").unwrap_or_else(|_| whoami::username());
        Self::connect(host, port, user)
    }

    fn connect(
        host: String,
        port: u16,
        user: String,
    ) -> Result<(ssh2::Session, ssh2::Sftp), String> {
        let addr = format!("{host}:{port}");
        let tcp = TcpStream::connect(&addr).map_err(|e| format!("连接失败 {addr}: {e}"))?;

        let mut session = ssh2::Session::new().map_err(|e| format!("创建 SSH 会话失败: {e}"))?;
        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| format!("SSH 握手失败: {e}"))?;

        if let Ok(pwd) = std::env::var("SFTP_PASSWORD") {
            session
                .userauth_password(&user, &pwd)
                .map_err(|e| format!("密码认证失败: {e}"))?;
        } else {
            let key_path = std::env::var("SFTP_KEY_PATH").unwrap_or_else(|_| {
                format!("{}/.ssh/id_rsa", std::env::var("HOME").unwrap_or_default())
            });
            session
                .userauth_pubkey_file(&user, None, Path::new(&key_path), None)
                .map_err(|e| format!("密钥认证失败 ({key_path}): {e}"))?;
        }

        let sftp = session
            .sftp()
            .map_err(|e| format!("SFTP 初始化失败: {e}"))?;
        Ok((session, sftp))
    }

    fn ensure_parent_dir(sftp: &ssh2::Sftp, path: &str) {
        if let Some(parent) = Path::new(path).parent() {
            let _ = sftp.mkdir(parent, 0o755);
        }
    }
}

#[async_trait]
impl StorageProvider for SftpProvider {
    fn name(&self) -> &'static str {
        "sftp"
    }

    async fn send(&self, local_path: &str, remote_path: &str) -> Result<String, String> {
        let host =
            std::env::var("SFTP_HOST").map_err(|_| "请设置 SFTP_HOST 环境变量".to_string())?;
        let port = std::env::var("SFTP_PORT")
            .unwrap_or_else(|_| "22".to_string())
            .parse::<u16>()
            .map_err(|_| "SFTP_PORT 格式错误".to_string())?;
        let user = std::env::var("SFTP_USER").unwrap_or_else(|_| whoami::username());
        let data = tokio::fs::read(local_path)
            .await
            .map_err(|e| format!("读取文件失败: {e}"))?;
        let data = tokio::fs::read(local_path)
            .await
            .map_err(|e| format!("读取文件失败: {e}"))?;
        let size = data.len();

        let remote = remote_path.to_string();
        let h = host.clone();
        let u = user.clone();
        tokio::task::spawn_blocking(move || {
            let (_session, sftp) = Self::connect(h, port, u)?;
            Self::ensure_parent_dir(&sftp, &remote);
            let mut file = sftp
                .create(Path::new(&remote))
                .map_err(|e| format!("创建远程文件失败: {e}"))?;
            file.write_all(&data)
                .map_err(|e| format!("写入远程文件失败: {e}"))?;
            Ok::<(), String>(())
        })
        .await
        .map_err(|e| format!("SFTP 上传失败: {e}"))?
        .map_err(|e| e)?;

        println!("✓ 已上传: {local_path} → {remote_path} ({size} 字节)");
        Ok(format!("sftp://{user}@{host}:{port}{remote_path}"))
    }

    async fn receive(&self, url: &str, local_path: &str) -> Result<(), String> {
        // sftp://user@host:port/path
        let user_host_path = url
            .strip_prefix("sftp://")
            .and_then(|s| s.split_once('/'))
            .ok_or_else(|| format!("不支持的 URL 格式: {url}"))?;

        let user_host = user_host_path.0.to_string();
        let remote_path = format!("/{}", user_host_path.1);

        let (user, host_with_port) = user_host
            .split_once('@')
            .ok_or_else(|| "URL 缺少 user@host".to_string())?;

        let (host, port_str) = host_with_port
            .split_once(':')
            .unwrap_or((host_with_port, "22"));
        let port: u16 = port_str.parse().map_err(|_| "端口格式错误".to_string())?;

        let user = user.to_string();
        let host = host.to_string();
        let local = local_path.to_string();

        tokio::task::spawn_blocking(move || {
            let (_session, sftp) = Self::connect(host, port, user)?;
            let mut file = sftp
                .open(Path::new(&remote_path))
                .map_err(|e| format!("打开远程文件失败: {e}"))?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| format!("读取远程文件失败: {e}"))?;
            std::fs::write(&local, &buf).map_err(|e| format!("写入本地文件失败: {e}"))?;
            println!("✓ 已接收: {local} ({} 字节)", buf.len());
            Ok::<(), String>(())
        })
        .await
        .map_err(|e| format!("SFTP 下载失败: {e}"))?
    }

    async fn receive_path(&self, remote: &str, local: &str) -> Result<(), String> {
        let host =
            std::env::var("SFTP_HOST").map_err(|_| "请设置 SFTP_HOST 环境变量".to_string())?;
        let port = std::env::var("SFTP_PORT")
            .unwrap_or_else(|_| "22".to_string())
            .parse::<u16>()
            .map_err(|_| "SFTP_PORT 格式错误".to_string())?;
        let user = std::env::var("SFTP_USER").unwrap_or_else(|_| whoami::username());
        let local_path = local.to_string();
        let remote_path = remote.to_string();

        tokio::task::spawn_blocking(move || {
            let (_session, sftp) = Self::connect(host, port, user)?;
            let mut file = sftp
                .open(Path::new(&remote_path))
                .map_err(|e| format!("打开远程文件失败: {e}"))?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|e| format!("读取远程文件失败: {e}"))?;
            std::fs::write(&local_path, &buf).map_err(|e| format!("写入本地文件失败: {e}"))?;
            println!("✓ 已接收: {local_path} ({} 字节)", buf.len());
            Ok::<(), String>(())
        })
        .await
        .map_err(|e| format!("SFTP 下载失败: {e}"))?
    }
}
