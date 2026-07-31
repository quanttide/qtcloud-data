# qtcloud-data CLI — 开发者文档

## 架构概览

```
src/
  main.rs           # 入口：CLI 参数解析，分发到各命令
  lib.rs            # 库入口：暴露所有模块
  transfer.rs       # 传输逻辑：--provider 选择、send/receive 分发
  process.rs        # 编排流程：receive → pipeline → send
  pipeline.rs       # 管道管理：list/show，shell 调用 cue
  blueprint.rs      # 蓝图管理：list/show，shell 调用 cue
  contract.rs       # 契约管理：list/show，支持 cue / yaml
  providers/
    mod.rs          # StorageProvider trait + 工厂函数
    dropbox.rs      # Dropbox 实现
    baidu_drive.rs  # 百度网盘实现
    google_drive.rs # Google Drive 实现
    onedrive.rs     # OneDrive 实现
    s3.rs           # S3 实现
    sftp.rs         # SFTP 实现
```

## 命令层级

| 命令 | 职责 | 依赖 |
|---|---|---|
| `transfer` | 原子传输操作（send/receive） | 各平台 API |
| `process` | 编排 receive → pipeline → send | 自身 shell 调用 transfer |
| `pipeline` | 管道定义查看 | `cue` 命令 |
| `blueprint` | 蓝图定义查看 | `cue` 命令 |
| `contract` | 契约定义查看 | `cue` 或直接文件读取 |

## StorageProvider trait

所有传输平台实现此 trait：

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

## process 编排

`process` 命令串联三步：

1. **receive** — shell 调用 `transfer receive`
2. **pipeline** — 顺序执行步骤链，每步输入 CSV 输出 CSV
3. **send** — shell 调用 `transfer send`

Pipeline 定义在 CUE 文件中，通过 `--pipeline` 或 `--blueprint` 引用。

## 添加新平台

新建 `providers/<name>.rs`，实现 `StorageProvider` trait，在 `providers/mod.rs` 注册。

### 认证约定

| Provider | 环境变量 |
|---|---|
| Dropbox | `DROPBOX_ACCESS_TOKEN` |
| 百度网盘 | `BAIDU_ACCESS_TOKEN` |
| Google Drive | `GOOGLE_DRIVE_ACCESS_TOKEN` |
| OneDrive | `ONEDRIVE_ACCESS_TOKEN` |
| S3 | `AWS_ACCESS_KEY_ID` + `AWS_SECRET_ACCESS_KEY` |
| SFTP | `SFTP_HOST` + `SFTP_USER` |

### 手动 vs 自动模式

- **手动**：`receive` 传入 URL，自动识别提供商。全部平台支持
- **自动**：`receive` 传入路径 + `--provider`。仅 S3、SFTP 等直接访问平台支持

## 环境变量

| 变量 | 默认值 | 用途 |
|---|---|---|
| `PIPELINE` | `csv-standard` | 默认 pipeline 名称 |
| `PIPELINES_DIR` | `./pipelines` | CUE 管道定义目录 |
| `BLUEPRINTS_DIR` | `./blueprints` | CUE 蓝图定义目录 |
| `CONTRACTS_DIR` | `./contracts` | 契约定义目录 |
| `WORKDIR` | `./work` | 流程执行工作目录 |
| `QTDATA_CLI` | `qtcloud-data` | 自身命令路径（process 调用） |

## 测试

```bash
cargo test
```

使用 `wiremock` 模拟 HTTP 响应。测试位于 `tests/integration_test.rs`。

## 构建

```bash
cargo build --release
```

产物：`target/release/qtcloud-data`
