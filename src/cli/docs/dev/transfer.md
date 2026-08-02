# qtcloud-data CLI — 开发者文档

## 架构概览

```
src/
  main.rs           # 入口：CLI 参数解析，run_command 分发到各命令
  lib.rs            # 库入口：暴露所有模块；test_support（测试共享 helper）
  error.rs          # CliError：统一错误类型，命令入口返回 Result<(), CliError>
  store.rs          # 统一路径解析（catalog_dir）、UTC 时间、JSON registry 读写（原子写盘）
  transfer.rs       # 传输逻辑：--provider 选择、send/receive 进程内服务函数
  process.rs        # 编排流程：StepExecutor（receive → pipeline → send）状态机
  pipeline.rs       # 管道管理：list/show
  blueprint.rs      # 蓝图管理：list/show
  blueprint_core.rs # 共享：prompt 模板、表格解析、HTML 渲染、目录解析
  contract.rs       # 契约管理：list/show（文件直读为主，cue 为可选增强）
  catalog.rs        # 数据目录：Volume 登记（registry.json），VolumeStatus 枚举
  spec.rs           # Specification YAML envelope：wrap / validate / load
  doctor.rs         # 环境检查：工具、目录、凭证
  clarify.rs        # LLM：聊天记录 → DRD（ClarifyHandler）
  design.rs         # LLM：DRD → Contract / Blueprint / Formalize / Preview（DesignHandler）
  implement.rs      # LLM：Blueprint → Python 实现（ImplementHandler）
  review.rs         # LLM：审计 Specification（ReviewHandler）
  version.rs        # 规格版本历史（git log/show/diff）
  providers/
    mod.rs          # StorageProvider trait + 工厂函数
    dropbox.rs      # Dropbox 实现（upload/create_shared_link 支持 mock base 注入）
    baidu_drive.rs  # 百度网盘实现
    google_drive.rs # Google Drive 实现（send_with_base/receive_with_base 支持 mock）
    onedrive.rs     # OneDrive 实现（send_with_base 支持 mock）
    s3.rs           # S3 实现（AWS_ENDPOINT_URL 可指向 mock）
    sftp.rs         # SFTP 实现
```

## 命令层级

| 命令 | 职责 | 依赖 |
|---|---|---|
| `clarify` | LLM 生成 DRD | LLM（`ClarifyHandler`） |
| `design` | LLM 生成 Contract/Blueprint | LLM（`DesignHandler`） |
| `review` | LLM 审计 Specification | LLM（`ReviewHandler`） |
| `implement` | LLM 生成 Python 实现 | LLM（`ImplementHandler`） |
| `spec` | Specification envelope wrap/validate | 无 |
| `version` | 规格版本历史 | git |
| `doctor` | 环境检查 | 外部工具探测 |
| `transfer` | 原子传输操作（send/receive） | 各平台 API |
| `process` | 编排 receive → pipeline → send | 库内组合 transfer 服务函数 |
| `pipeline` | 管道定义查看 | cue（v0.2.2 计划改文件直读为主） |
| `blueprint` | 蓝图定义查看 | cue（v0.2.2 计划改文件直读为主） |
| `contract` | 契约定义查看 | 文件直读（cue 可选） |
| `catalog` | 数据目录登记 | 无 |

## 错误模型（v0.2.1）

所有命令入口返回 `Result<(), CliError>`，`main` 顶层统一格式化 `错误: {err}` 并退出码 1。

```rust
// 命令实现（以 catalog 为例）
pub fn run(args: &CatalogArgs) -> Result<(), CliError> {
    match &args.action {
        CatalogAction::Show { name } => show(name),
        // ...
    }
}

fn show(name: &str) -> Result<(), CliError> {
    match registry.get(name) {
        Some(v) => { /* 打印 */ Ok(()) }
        None => Err(CliError::new(format!("未找到 volume: {name}"))),
    }
}
```

- 错误路径通过 `Result` 传播，**不直接 `std::process::exit(1)`**（仅 `main` 保留 bin 入口 exit）
- `Result<_, String>` 的公开函数已收敛为 `CliError`（`From<io::Error>/String/&str`）
- 错误路径因此可测试：`cmd_xxx(...).unwrap_err()`

## LLM 命令注入（v0.2.1）

LLM 命令（clarify/design/implement/review）使用 **Handler 构造器注入**：

```rust
pub struct DesignHandler {
    llm: quanttide_agent::LLM,
}

impl DesignHandler {
    pub fn new(llm: quanttide_agent::LLM) -> Self { Self { llm } }
    pub fn run(&self, args: &DesignArgs) -> Result<(), CliError> { /* ... */ }
}
```

- 生产路径：`main.rs` 里 `DesignHandler::new(quanttide_agent::LLM::default())`
- 测试路径：`lib.rs test_support::fake_llm(content)` 复用 quanttide-agent 的 `HttpClient` 抽象构造假 LLM，不发起网络请求

## store 模块（v0.2.1）

`store.rs` 统一三处曾经重复的拷贝：

- `catalog_dir()`：路径解析优先级 `CATALOG_DIR` > `DATA_ROOT/catalog` > `.quanttide/data/catalog`
- `now_utc()`：UTC 时间格式化
- `Registry<T>`：JSON registry 读写（insert/remove/save），**原子写盘**（临时文件 + rename）

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

`transfer send` 成功后会把 provider、源文件、远程路径、分享链接和可选 `--output` 链接文件路径写入 `CATALOG_DIR/delivery-links.json`。记录失败只输出 warning，不影响已经成功的上传和链接输出。

Provider 实现遵循 **mock 支持约定**：需要 mock 的平台抽 `*_with_base` / `upload` / `create_shared_link` 等公开 helper，接受 `Option<&str>` 端点参数（dropbox / google_drive / onedrive 已支持，s3 通过 `AWS_ENDPOINT_URL`）。

## process 编排

`process` 命令通过 **StepExecutor** 状态机串联三步（库内组合，不再自我 re-exec）：

1. **receive** — `transfer::receive`（进程内，QTDATA_CLI env 时委派外部 CLI）
2. **pipeline** — 顺序执行步骤链，每步输入 CSV 输出 CSV
3. **send** — `transfer::send`（进程内）

Pipeline 定义在 CUE 文件中，通过 `--pipeline` 或 `--blueprint` 引用。

`process` 会把 job 记录追加/覆盖到 `CATALOG_DIR/jobs.json`。该文件沿用 catalog 的 pretty JSON registry 风格，顶层是以 job id 为 key 的对象；每条记录包含 `customer_id`、脱敏后的 `source_url`、`blueprint`、`pipeline`、`raw_path`、`output_path`、`link_path`、`status`、`started_at`、`finished_at` 和 `log_path`。对应日志写在 `CATALOG_DIR/jobs/<job-id>.log`。

成功交付后，`process` 会通过 catalog 的登记逻辑把最终产物写入 `CATALOG_DIR/registry.json`，volume 的 `provider` 为 `process`，`source` 为 `process:<job-id>`，`status` 为 `delivered`。登记失败只输出 warning，不反转已经完成的交付结果。

## 添加新平台

新建 `providers/<name>.rs`，实现 `StorageProvider` trait，在 `providers/mod.rs` 注册。对需要 mock 的平台，遵循 `*_with_base` 注入约定并补 wiremock 测试。

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
| `PIPELINE_DIR` | `.quanttide/data/pipeline` | 管道定义目录 |
| `BLUEPRINT_DIR` | `.quanttide/data/blueprint` | 蓝图定义目录 |
| `CONTRACT_DIR` | `.quanttide/data/contract` | 契约定义目录 |
| `DATA_ROOT` | `.quanttide/data` | 数据根目录（覆盖各子目录默认值） |
| `WORKDIR` | 系统临时目录下的 `qtcloud-data` | 流程执行工作目录 |
| `QTDATA_CLI` | `qtcloud-data` | 自身命令路径（transfer 委派逃生舱） |
| `CATALOG_DIR` | `.quanttide/data/catalog` | catalog registry、process job 记录和 delivery link 记录目录 |

## 测试

```bash
cargo test          # 全部测试
cargo clippy --locked -- -D warnings   # CI 严格 lint
cargo fmt --check   # 格式检查
```

测试分层：

| 层次 | 位置 | 手段 |
|---|---|---|
| 单元测试 | `src/*.rs` 内 `mod tests` | 纯函数 + env 注入 + `test_support::{temp_dir, write_script, fake_llm}` |
| 集成测试 | `tests/blueprint_test.rs`、`tests/integration_test.rs`、`tests/provider_test.rs` | spawn 二进制、wiremock 模拟 HTTP |
| e2e | `tests/e2e_baseline.rs` | 真实 fixture 全链路（`tests/fixtures/github-activity/`） |

覆盖率：`cargo llvm-cov test --workspace`（当前 83.7%）。CI 在 `RUSTFLAGS=-D warnings` 下运行。

## 构建

```bash
cargo build --release
```

产物：`target/release/qtcloud-data`
