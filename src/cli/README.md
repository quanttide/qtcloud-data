# qtcloud-data CLI

量潮数据云命令行工具。

## 命令一览

| 命令 | 用途 |
|---|---|
| `transfer` | 数据传输（send / receive），支持 6 个平台 |
| `process` | 编排流程（receive → pipeline → send） |
| `pipeline` | 管道定义查看（list / show） |
| `blueprint` | 蓝图定义查看（list / show，组合 contract + pipeline） |
| `contract` | 契约定义查看（list / show） |

## 安装

```bash
cd apps/qtcloud-data/src/cli
cargo build --release
```

产物在 `target/release/qtcloud-data`。

## 快速开始

```bash
# 设置凭证
export DROPBOX_ACCESS_TOKEN=你的token

# 发送文件 → 生成分享链接
qtcloud-data transfer send ./file.pdf

# 接收文件 ← 共享链接
qtcloud-data transfer receive "https://www.dropbox.com/s/xxx/file.pdf"

# 按 blueprint 执行完整流程
qtcloud-data process ABC "https://..." --blueprint csv-standardization
```

## 文档

- [用户文档](docs/user/transfer.md) — 安装、认证、命令用法
- [开发者文档](docs/dev/transfer.md) — 架构、添加新平台、测试

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
