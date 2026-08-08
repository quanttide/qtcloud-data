# Changelog

## [studio/v0.1.0-alpha.2] - 2026-08-08

### 新增
- deploy-studio CI 工作流：`studio/*` tag 触发构建，OSS 上传 + CDN 刷新（data.cloud.quanttide.com）
- Terraform IaC（`infra/studio/`）：OSS 桶 + CDN 域名定义
- 脱敏版部署运维记录（`docs/dev-guide/static-site-ops.md`）

### 修复
- OSS 桶开放公共读并开启静态网站托管（关闭新桶默认的桶级 BlockPublicAccess 后生效）

## [studio/v0.1.0-alpha.1] - 2026-08-08

### 新增
- 量潮数据云控制台首个 alpha 版本（原型阶段）
- 蓝图（Blueprint）页面：列表浏览与详情查看方向规划（见 ROADMAP.md）
- deploy-studio CI 工作流：`studio/*` tag 触发构建，部署到 OSS 并刷新 CDN（data.cloud.quanttide.com）

### 修复
- Flutter 3.24 兼容：`withValues` 替换为 `withOpacity`
