# qtcloud-data CLI — 用户文档

量潮数据云命令行工具。

## 安装

```bash
cd apps/qtcloud-data/src/cli
cargo build --release
```

产物在 `target/release/qtcloud-data`。

## 认证

```bash
export DROPBOX_ACCESS_TOKEN=你的token
# 或 export BAIDU_ACCESS_TOKEN / GOOGLE_DRIVE_ACCESS_TOKEN / ONEDRIVE_ACCESS_TOKEN
# 或 AWS 凭证链（S3）/ SFTP_HOST（SFTP）
```

## 命令

### transfer — 数据传输

原子操作：上传文件生成分享链接（send），或从链接下载（receive）。

```bash
# 发送文件 → 生成分享链接
qtcloud-data transfer send ./report.pdf
# 输出: https://www.dropbox.com/s/abc/report.pdf?dl=1

# 接收文件 ← 分享链接（手动模式）
qtcloud-data transfer receive "https://dropbox.com/s/xxx/data.csv"

# 自动拉取（S3/SFTP，无需 URL）
qtcloud-data transfer receive /remote/data.csv --provider s3
```

支持平台：`dropbox`（默认）| `baidu` | `google` | `onedrive` | `s3` | `sftp`

### process — 编排流程

串联 receive → pipeline → send 三步骤。pipelines 目录支持 CUE 定义。

```bash
# 默认 pipeline（csv-standard）
qtcloud-data process ABC-001 "https://dropbox.com/s/xxx/data.csv"

# 指定 pipeline
qtcloud-data process ABC-001 "https://..." --pipeline csv-standard

# 按 blueprint 运行
qtcloud-data process ABC-001 "https://..." --blueprint csv-standardization
```

环境变量：`PIPELINE`（默认 pipeline）、`QTDATA_CLI`、`WORKDIR`

### pipeline — 管道管理

查看可用的数据处理管道定义。

```bash
qtcloud-data pipeline list
# csv-standard, annotation

qtcloud-data pipeline show csv-standard
# 显示步骤详情：processor → enricher
```

环境变量：`PIPELINES_DIR`（默认 `./pipelines`，需 `cue`）

### blueprint — 蓝图管理

查看可用的蓝图定义（契约 + 管道 + 验收规则）。

```bash
qtcloud-data blueprint list
# CSV 数据标准化, 问卷清洗

qtcloud-data blueprint show csv-standardization
# 显示契约、管道、验收规则
```

环境变量：`BLUEPRINTS_DIR`（默认 `./blueprints`，需 `cue`）

### contract — 契约查看

查看独立的数据契约定义（schema、格式等）。

```bash
qtcloud-data contract list
qtcloud-data contract show source-questionnaire
```

环境变量：`CONTRACTS_DIR`（默认 `./contracts`，支持 .yaml / .cue / .json）

## 命令关系

```
transfer send/receive    — 原子传输操作
process                  — 编排：receive → pipeline → send
pipeline list/show       — 管道定义查看
blueprint list/show      — 蓝图定义查看（组合 contract + pipeline）
contract list/show       — 契约定义查看
```

## 跨平台传输示例

```bash
# 接收百度网盘 → 处理 → 用 S3 交付
qtcloud-data transfer receive "https://pan.baidu.com/s/xxx"
qtcloud-data transfer send ./result.csv --provider s3

# 完整编排（需配置 blueprints/ 和 pipelines/）
qtcloud-data process ABC "https://dropbox.com/s/xxx/data.csv"
```
