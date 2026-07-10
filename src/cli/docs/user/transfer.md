# qtcloud-data transfer — 用户文档

`qtcloud-data transfer` 是基于网盘的数据传输工具。支持从共享链接接收文件，或将本地文件发出去并生成分享链接。

## 安装

```bash
cd apps/qtcloud-data/src/cli
cargo build --release
```

产物在 `target/release/qtcloud-data`。

## 认证

使用前需设置对应平台的访问凭证：

```bash
# Dropbox（默认）
export DROPBOX_ACCESS_TOKEN=你的token

# 百度网盘
export BAIDU_ACCESS_TOKEN=你的token

# Google Drive
export GOOGLE_DRIVE_ACCESS_TOKEN=你的token

# OneDrive
export ONEDRIVE_ACCESS_TOKEN=你的token

# Amazon S3（标准 AWS 凭证链）
export AWS_ACCESS_KEY_ID=xxx
export AWS_SECRET_ACCESS_KEY=xxx
export AWS_REGION=us-east-1
export S3_BUCKET=my-bucket
```

## 命令

### 发送文件

上传本地文件到平台并生成分享链接：

```bash
qtcloud-data transfer send ./report.pdf
# 输出: https://www.dropbox.com/s/abc123/report.pdf?dl=1
```

指定提供商：

```bash
qtcloud-data transfer send ./report.pdf --provider baidu
qtcloud-data transfer send ./report.pdf --provider s3
```

指定远程路径：

```bash
qtcloud-data transfer send ./data.csv /Customers/ABC/result.csv
```

将链接保存到文件（而非打印到终端）：

```bash
qtcloud-data transfer send ./report.pdf --output link.txt
```

### 接收文件

支持两种模式：

#### 手动模式 — 从分享链接下载

传入 `http://` / `https://` 开头的 URL，自动识别提供商：

```bash
qtcloud-data transfer receive "https://www.dropbox.com/s/xxx/data.csv"
```

指定保存路径：

```bash
qtcloud-data transfer receive "https://drive.google.com/file/d/xxx/view" --output ./incoming/data.csv
```

URL 自动识别：

| 链接格式 | 自动识别 |
|---|---|
| `dropbox.com/s/...` | Dropbox |
| `pan.baidu.com/s/...` | 百度网盘 |
| `drive.google.com/file/d/...` | Google Drive |
| `1drv.ms/...` / `onedrive.live.com/...` | OneDrive |
| `s3.amazonaws.com/...` 或 presigned URL | S3 |

#### 自动模式 — 直接从存储拉取

传入远程路径（非 URL），配合 `--provider` 指定平台。无需传 URL，适合内部系统对接：

```bash
qtcloud-data transfer receive /Customers/ABC/data.csv --provider s3
```

> 自动模式目前仅 S3 支持。网盘类平台（Dropbox、百度等）调用自动模式会提示"不支持自动接收，请提供分享链接"。

## 示例

### 与客户交换数据（手动模式）

```bash
# 1. 接收客户发来的文件
qtcloud-data transfer receive "https://www.dropbox.com/s/xxx/customer_data.csv"

# 2. 处理...

# 3. 发送结果给客户
qtcloud-data transfer send ./result.csv --output link.txt
# 把 link.txt 里的链接发给客户即可下载
```

### 内部系统自动传输（自动模式）

```bash
# 处理结果自动上传到 S3
qtcloud-data transfer send ./result.csv --provider s3

# 另一端直接拉取（无需传 URL）
qtcloud-data transfer receive /output/result.csv --provider s3
```

### 跨平台传输

```bash
# 从百度网盘接收
qtcloud-data transfer receive "https://pan.baidu.com/s/xxx"

# 处理后用 Google Drive 发出去
qtcloud-data transfer send ./result.csv --provider google
```

## 支持的平台

| 平台 | `--provider` 值 | 认证方式 | 自动接收 |
|---|---|---|---|
| Dropbox | `dropbox`（默认） | OAuth 2.0 token | ❌ |
| 百度网盘 | `baidu` | OAuth 2.0 token | ❌ |
| Google Drive | `google` | OAuth 2.0 token | ❌ |
| OneDrive | `onedrive` | OAuth 2.0 token | ❌ |
| S3 | `s3` | AWS 凭证链 | ✅ |

## 常见问题

**Q: 提示"请设置 XXX_ACCESS_TOKEN 环境变量"**

A: 需要先在对应平台创建应用并获取 OAuth 2.0 access token，然后设置为环境变量。

**Q: 如何获取 access token？**

A: 各平台 OAuth 流程：

- Dropbox：https://www.dropbox.com/developers/apps → 创建应用 → 生成 access token
- 百度网盘：https://pan.baidu.com/union/ → 创建应用 → OAuth 2.0 授权码流程
- Google Drive：https://console.cloud.google.com/ → 启用 Drive API → 创建 OAuth 2.0 凭据
- OneDrive：https://portal.azure.com/ → 注册应用 → 启用 Microsoft Graph API 权限
- S3：标准 AWS 凭证（环境变量 / ~/.aws/credentials / IAM 角色）
