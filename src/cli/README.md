# qtcloud-data CLI

量潮数据云命令行工具，用来把数据交付里容易卡住的碎片环节串成可重复命令。

当前 crates.io 发布准备版本：`qtcloud-data-cli` v0.2.0。
v0.2.0 补齐 `doctor`、`process` job 记录、catalog 产物登记和 Provider 对齐能力。

## 命令一览

| 命令 | 用途 |
|---|---|
| `clarify` | 从客户聊天记录或上下文生成 DRD 数据需求文档 |
| `design` | 从 DRD 生成 Contract / Blueprint Specification（YAML + MD + HTML） |
| `spec` | 固化 Specification YAML envelope（wrap / validate） |
| `implement` | 从 Blueprint YAML 生成 Python 代码实现 |
| `review` | 审计 DRD 或 Specification 的完整性和一致性 |
| `version` | 查看和比较规格版本 |
| `transfer` | 数据传输（send / receive），支持 6 个平台 |
| `doctor` | 检查本机 DataOps 环境、目录、工具和常见凭证 |
| `process` | 编排流程（receive → pipeline → send） |
| `pipeline` | 管道定义查看（list / show） |
| `blueprint` | 蓝图定义查看（list / show，组合 contract + pipeline） |
| `contract` | 契约定义查看（list / show） |
| `catalog` | 管理本地数据目录和文件登记 |

## 安装

### crates.io

```bash
cargo install qtcloud-data-cli
```

### 本地开发

```bash
cd src/cli
cargo build
```

调试产物在 `target/debug/qtcloud-data`，发布产物在 `target/release/qtcloud-data`。

### 源码交付安装

CLI + Provider 源码交付包中只需要保留 `src/cli` 和 `src/provider`。CLI 的内部依赖使用 crates.io 版本依赖，不依赖开发者本机的 `D:\packages` 目录。

```bash
cd src/cli
cargo install --path .
qtcloud-data --help
```

### 二进制包

GitHub Release `cli/v0.2.0` 待发布，计划提供：

- `qtcloud-data-x86_64-unknown-linux-gnu.tar.gz`
- `qtcloud-data-x86_64-pc-windows-msvc.zip`

## 快速开始

```bash
# 设置凭证
export DROPBOX_ACCESS_TOKEN=你的token

# 检查本机环境
qtcloud-data doctor --no-fail
qtcloud-data doctor --fix-dirs --json --no-fail

# 从聊天记录生成需求文档
qtcloud-data clarify from-chat ./context.md

# 从需求文档生成规格书
qtcloud-data design contract .quanttide/data/drd/context.md
qtcloud-data design blueprint .quanttide/data/drd/context.md

# 从规格书生成 Python 实现
qtcloud-data implement .quanttide/data/spec/context-blueprint.yaml --lang python

# 将旧 Blueprint YAML 包装成稳定 Specification envelope
qtcloud-data spec wrap .quanttide/data/spec/context-blueprint.yaml
qtcloud-data spec validate .quanttide/data/spec/context-spec.yaml

# 发送文件 → 生成分享链接
qtcloud-data transfer send ./file.pdf
qtcloud-data transfer send ./file.pdf --output .quanttide/data/catalog/share-link.txt

# 接收文件 ← 共享链接
qtcloud-data transfer receive "https://www.dropbox.com/s/xxx/file.pdf"

# 按 blueprint 执行完整流程
qtcloud-data process ABC "https://..." --blueprint csv-standardization
```

`design blueprint` 生成的 YAML 会同时包含兼容旧实现的 `pipeline.steps` 和 Step Functions 风格的 `pipeline.start_at` / `pipeline.states`。v0.2.0 会给每个步骤默认写入 `resource: builtin:copy`，方便 Provider 先做端到端 smoke test；实际业务实现生成后，再把 `resource` 替换为 `python:<script>` 等真实处理脚本。

本地源码构建版本中，`process` 执行后会在 `CATALOG_DIR/jobs.json` 写入 job 记录，并在 `CATALOG_DIR/jobs/` 下生成对应日志文件。记录包含客户、来源、blueprint、pipeline、原始文件、最终结果、分享链接文件、状态和日志路径。

`transfer send` 成功后会把交付链接记录到 `CATALOG_DIR/delivery-links.json`。使用 `--output` 时，链接仍会同时写入指定文件。

成功交付时，`process` 还会把最终产物登记到 `CATALOG_DIR/registry.json`，provider 为 `process`，source 为 `process:<job-id>`，status 为 `delivered`。

## DataOps 目录

默认目录根是 `.quanttide/data`，也可以用 `DATA_ROOT` 调整。单个目录仍可用对应环境变量覆盖。

| 用途 | 默认路径 | 环境变量 |
|---|---|---|
| 数据根目录 | `.quanttide/data` | `DATA_ROOT` |
| 需求文档 | `.quanttide/data/drd` | `DRD_DIR` |
| Specification | `.quanttide/data/spec` | `SPEC_DIR` |
| Blueprint | `.quanttide/data/blueprint` | `BLUEPRINT_DIR` |
| Contract | `.quanttide/data/contract` | `CONTRACT_DIR` |
| Pipeline | `.quanttide/data/pipeline` | `PIPELINE_DIR` |
| Catalog | `.quanttide/data/catalog` | `CATALOG_DIR` |

## 文档

- [用户文档](docs/user/transfer.md) — 安装、认证、命令用法
- [开发者文档](docs/dev/transfer.md) — 架构、添加新平台、测试
- [Specification 契约](docs/dev/specification.md) — CLI 与 Provider 共用的 YAML envelope
- [发布流程](docs/dev/release.md) — `qtcloud-devops` 预检、GitHub Actions 和 crates.io 发布约定
- [ROADMAP](ROADMAP.md) — 版本里程碑和 checkbox 计划
- [TODO](TODO.md) — 按模块拆解的执行任务

## 支持的传输平台

| 平台 | `--provider` | 环境变量 |
|---|---|---|
| Dropbox（默认） | `dropbox` | `DROPBOX_ACCESS_TOKEN` |
| 百度网盘 | `baidu` | `BAIDU_ACCESS_TOKEN` |
| Google Drive | `google` | `GOOGLE_DRIVE_ACCESS_TOKEN` |
| OneDrive | `onedrive` | `ONEDRIVE_ACCESS_TOKEN` |
| S3 | `s3` | AWS 凭证链 |
| SFTP | `sftp` | `SFTP_HOST` + `SFTP_USER` |

## 许可

MIT
