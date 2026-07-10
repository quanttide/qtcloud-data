# qtcloud-data transfer — 用户文档

`qtcloud-data transfer` 是基于网盘的数据传输工具。支持从共享链接接收文件，或将本地文件发出去并生成分享链接。

## 安装

```bash
cd apps/qtcloud-data/src/cli
cargo build --release
```

产物在 `target/release/qtcloud-data`。

## 认证

使用前需设置对应网盘的访问凭证：

```bash
# Dropbox（默认）
export DROPBOX_ACCESS_TOKEN=你的token

# 百度网盘
export BAIDU_ACCESS_TOKEN=你的token

# Google Drive
export GOOGLE_DRIVE_ACCESS_TOKEN=你的token

# OneDrive
export ONEDRIVE_ACCESS_TOKEN=你的token
```

## 命令

### 发送文件

上传本地文件到网盘并生成分享链接：

```bash
qtcloud-data transfer send ./report.pdf
# 输出: https://www.dropbox.com/s/abc123/report.pdf?dl=1
```

指定提供商：

```bash
qtcloud-data transfer send ./report.pdf --provider baidu
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

从分享链接下载文件到本地：

```bash
qtcloud-data transfer receive "https://www.dropbox.com/s/xxx/data.csv"
# 输出: ✓ 已接收: data.csv (xxxx 字节)
```

指定保存路径：

```bash
qtcloud-data transfer receive "https://drive.google.com/file/d/xxx/view" --output ./incoming/data.csv
```

URL 自动识别提供商，无需 `--provider` 参数：

| 链接格式 | 自动识别 |
|---|---|
| `dropbox.com/s/...` | Dropbox |
| `pan.baidu.com/s/...` | 百度网盘 |
| `drive.google.com/file/d/...` | Google Drive |
| `1drv.ms/...` / `onedrive.live.com/...` | OneDrive |

## 示例

### 与客户交换数据

```bash
# 1. 接收客户发来的文件
dropbox receive "https://www.dropbox.com/s/xxx/customer_data.csv"

# 2. 处理...

# 3. 发送结果给客户
dropbox send ./result.csv --output link.txt
# 把 link.txt 里的链接发给客户即可下载
```

### 跨网盘传输

```bash
# 从百度网盘接收
dropbox receive --provider baidu "https://pan.baidu.com/s/xxx"

# 处理后用 Google Drive 发出去
dropbox send ./result.csv --provider google
```

## 支持的平台

| 平台 | `--provider` 值 | 认证方式 |
|---|---|---|
| Dropbox | `dropbox`（默认） | OAuth 2.0 token |
| 百度网盘 | `baidu` | OAuth 2.0 token |
| Google Drive | `google` | OAuth 2.0 token |
| OneDrive | `onedrive` | OAuth 2.0 token |

## 常见问题

**Q: 提示"请设置 XXX_ACCESS_TOKEN 环境变量"**

A: 需要先在对应平台创建应用并获取 OAuth 2.0 access token，然后设置为环境变量。

**Q: 如何获取 access token？**

A: 各平台 OAuth 流程：

- Dropbox：https://www.dropbox.com/developers/apps → 创建应用 → 生成 access token
- 百度网盘：https://pan.baidu.com/union/ → 创建应用 → OAuth 2.0 授权码流程
- Google Drive：https://console.cloud.google.com/ → 启用 Drive API → 创建 OAuth 2.0 凭据
- OneDrive：https://portal.azure.com/ → 注册应用 → 启用 Microsoft Graph API 权限
