# 传输（transfer.rs + providers/）

本文档对应 `src/transfer.rs` 与 `src/providers/`。

## 职责

`transfer` 命令提供原子传输操作：`send`（上传 → 分享链接）与 `receive`（下载）。
支持 6 个平台，通过 `--provider` 选择。

## 进程内服务函数

v0.2.1 起 `send` / `receive` 抽为**进程内服务函数**（供 `process` 编排直接组合，替代自我 re-exec）：

```rust
pub fn receive(source: &str, output: &Path, provider: &str) -> Result<(), CliError>;
pub fn send(file: &str, remote: Option<&str>, output: Option<&Path>, provider: &str)
    -> Result<String, CliError>;   // 返回交付链接
```

- **委派逃生舱**：`QTDATA_CLI` 环境变量设置时，委派给外部 CLI（`transfer receive/send` 子命令）——测试与部署场景使用
- **进程内路径**：默认走进程内 provider（tokio runtime + StorageProvider）
- 错误类型收敛为 `CliError`

## StorageProvider trait

```rust
#[async_trait]
pub trait StorageProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn send(&self, local_path: &str, remote_path: &str) -> Result<String, String>;
    async fn receive(&self, url: &str, local_path: &str) -> Result<(), String>;
    async fn receive_path(&self, remote: &str, local: &str) -> Result<(), String> {
        Err("该平台不支持自动接收".to_string())
    }
}
```

### 手动 vs 自动模式

- **手动**：`receive` 传入 URL，自动识别提供商（`providers::detect`）。全部平台支持
- **自动**：`receive` 传入路径 + `--provider`。仅 S3、SFTP 等直接访问平台支持

## 平台实现与 mock 支持

| 平台 | `--provider` | 认证环境变量 | mock 方式 |
|------|-------------|-------------|-----------|
| Dropbox（默认） | `dropbox` | `DROPBOX_ACCESS_TOKEN` | `upload` / `create_shared_link` 接受 `mock_base: Option<&str>` |
| 百度网盘 | `baidu` | `BAIDU_ACCESS_TOKEN` | 未支持（需真实 API） |
| Google Drive | `google` | `GOOGLE_DRIVE_ACCESS_TOKEN` | `send_with_base` / `receive_with_base` 注入端点 |
| OneDrive | `onedrive` | `ONEDRIVE_ACCESS_TOKEN` | `send_with_base` 注入端点 |
| S3 | `s3` | AWS 凭证链 | `AWS_ENDPOINT_URL` 指向 mock |
| SFTP | `sftp` | `SFTP_HOST` + `SFTP_USER` | 未支持（需真实服务） |

**mock 约定**：需要 mock 的平台抽 `*_with_base` / `upload` / `create_shared_link` 等公开 helper，
接受 `Option<&str>` 端点参数；trait impl 传 `None` 走线上端点。测试用 wiremock 指向本地。

## 交付链接记录

`send` 成功后把记录写入 `CATALOG_DIR/delivery-links.json`（字段定义见 [data-format.md](data-format.md)）。
记录失败只输出 warning，不影响已经成功的上传和链接输出。

## 添加新平台

1. 新建 `providers/<name>.rs`，实现 `StorageProvider` trait
2. 在 `providers/mod.rs` 注册（`from_name` / `detect`）
3. 认证环境变量约定见上表
4. 需要 mock 的平台遵循 `*_with_base` 注入约定并补 wiremock 测试（`tests/provider_test.rs` 参考）
