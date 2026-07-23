# qtcloud-data 发布状态

> 生成日期: 2026-07-17 | 仓库: [quanttide/qtcloud-data](https://github.com/quanttide/qtcloud-data)

## 总览

| 模块 | 语言 | 版本 | 最后发布 | 状态 |
|------|------|------|----------|------|
| cli | Rust | v0.0.5 | 2026-07-10 | 活跃迭代 |
| provider | Go | v0.0.1 | 2026-07-10 | 骨架阶段 |
| studio | Flutter | - | 无 CHANGELOG | 骨架阶段 |

---

## cli (Rust)

### v0.0.5 (2026-07-10)

**Changed**
- 默认目录改为 `.quanttide/data/` 规范：pipelines / blueprints / contracts
- WORKDIR 改为系统临时目录 `/tmp/qtcloud-data`，用完自动清理
- README 更新：添加命令一览表、S3/SFTP 平台、process 示例
- Cargo.toml 版本更新为 0.0.5

### v0.0.4 (2026-07-10)

**Added**
- `process` 命令：编排 receive → pipeline → send 完整流程
- `pipeline list|show` 命令：查看 CUE 定义的管道
- `blueprint list|show` 命令：查看 CUE 定义的蓝图
- `contract list|show` 命令：查看独立契约定义
- 全覆盖测试（9 个用例）

**Changed**
- Cargo.toml 版本更新为 0.0.4

### v0.0.3 (2026-07-10)

**Added**
- S3 provider（`--provider s3`）：PutObject + 预签名 URL + 自动接收
- SFTP provider（`--provider sftp`）：密钥/密码认证，支持 `sftp://` URL 和自动模式
- receive 双模式：手动（URL）和自动（路径）

**Changed**
- Cargo.toml 版本更新为 0.0.3

### v0.0.2 (2026-07-10)

**Added**
- StorageProvider trait 架构，统一 send/receive 接口
- 百度网盘支持（`--provider baidu`）
- Google Drive 支持（`--provider google`）
- OneDrive 支持（`--provider onedrive`）
- wiremock 集成测试（6 个用例覆盖 send/receive/错误处理）
- 开发者文档（`docs/dev/transfer.md`）
- 用户文档（`docs/user/transfer.md`）

**Changed**
- CLI 增加 `--provider` 选项（默认 `dropbox`），receive 时自动从 URL 识别提供商
- 重构 dropbox 模块为 provider 模式
- Cargo.toml 版本从 0.1.0 更新为 0.0.2

**Removed**
- 夸克网盘支持（无官方 API）

### v0.0.1 (2026-07-10)

**Added**
- 初始版本：Dropbox 数据传输 CLI
- `transfer send`：上传文件到网盘并生成分享链接
- `transfer receive`：从共享链接下载文件
- `transfer ls`：列出网盘中的客户目录
- wiremock 集成测试框架

---

## provider (Go)

### v0.0.1 (2026-07-10)

**Added**
- Provider 接口定义（对应 Rust StorageProvider trait）
- Dropbox 传输实现
- S3 传输实现（stub）
- HTTP API 骨架（4 个端点）
- Pipeline 执行引擎（stub）
- 内存存储（process job 记录）
- Go 项目结构和模块化架构

---

## studio (Flutter)

无 CHANGELOG。项目已搭建多端骨架（Web/macOS/Windows/Linux/iOS/Android）。

当前页面：
- Dashboard
- Pipelines
- Blueprints

---

## 关键观察

1. **全部 pre-1.0**：三个模块均未达到稳定版本
2. **cli 迭代最快**：一天内连发 5 个版本，功能相对完整（多网盘 + 编排）
3. **provider/studio 滞后**：provider 有接口定义但实现多为 stub；studio 仅有页面骨架
4. **最后一次活动**：三个模块均在 2026-07-10（一周前）
