# qtcloud-data transfer — 开发者文档

## 架构概览

```
src/
  main.rs           # 入口：CLI 参数解析，分发到 transfer
  lib.rs            # 库入口：暴露 providers 和 transfer
  transfer.rs       # 传输逻辑：--provider 选择、send/receive 分发
  providers/
    mod.rs          # StorageProvider trait + 工厂函数
    dropbox.rs      # Dropbox 实现
    baidu_drive.rs  # 百度网盘实现
    google_drive.rs # Google Drive 实现
    onedrive.rs     # OneDrive 实现
    s3.rs           # S3 实现
```

## StorageProvider trait

所有平台提供商实现 `StorageProvider` trait：

```rust
#[async_trait]
pub trait StorageProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn send(&self, local_path: &str, remote_path: &str) -> Result<String, String>;
    async fn receive(&self, url: &str, local_path: &str) -> Result<(), String>;

    /// 自动接收：直接从远程路径拉取（S3、SFTP 等支持）
    async fn receive_path(&self, remote: &str, local: &str) -> Result<(), String> {
        Err("该平台不支持自动接收，请提供分享链接".to_string())
    }
}
```

- `send`：上传本地文件 → 创建分享链接 → 返回可分享的 URL
- `receive`：从分享链接下载文件到本地（手动模式）
- `receive_path`：直接从远程路径拉取（自动模式），S3 等需直接访问权限的平台重写此方法

## 添加新平台

新建 `providers/<name>.rs`，实现 `StorageProvider` trait，然后在 `providers/mod.rs` 注册：

1. 文件注册：`pub mod <name>;` + `pub use`
2. 名称注册：在 `from_name()` 中添加 `"<name>" => Some(Box::new(<Provider>))`
3. URL 检测：在 `detect()` 中添加 URL 域名匹配

### 认证约定

每个 provider 从独立的环境变量读取凭证：

| Provider | 环境变量 |
|---|---|
| Dropbox | `DROPBOX_ACCESS_TOKEN` |
| 百度网盘 | `BAIDU_ACCESS_TOKEN` |
| Google Drive | `GOOGLE_DRIVE_ACCESS_TOKEN` |
| OneDrive | `ONEDRIVE_ACCESS_TOKEN` |
| S3 | `AWS_ACCESS_KEY_ID` + `AWS_SECRET_ACCESS_KEY` |

### 手动模式 vs 自动模式

- **手动模式**：`receive` 传入 URL（以 `http://` 或 `https://` 开头），自动识别提供商。全部平台支持
- **自动模式**：`receive` 传入远程路径，配合 `--provider` 使用。仅 S3 等有直接访问权限的平台支持。网盘类默认返回错误

## 测试

### 单元测试

```bash
cargo test
```

使用 `wiremock` 模拟 HTTP 响应，不依赖真实网络。测试位于 `tests/integration_test.rs`。

### 测试新 provider

```rust
#[tokio::test]
async fn test_my_provider_send() {
    let server = MockServer::start().await;
    let base = server.uri();

    Mock::given(method("POST"))
        .and(path("/upload"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    // 测试逻辑...
}
```

## 构建

```bash
cargo build --release
```

产物：`target/release/qtcloud-data`
