# Changelog

## [studio/v0.1.0-beta.1] - 2026-08-08

### 重构
- 总览页改为系统汇总：按 spec 四层框架（需求→规格→实现→执行→交付）统计各模块并展示最近执行
- 导航栏遵循 spec 流程链排序：总览→需求→契约→蓝图→管道→执行→传输
- 前端独立化：总览/传输等页面改用 seed 数据，客户端不依赖服务端（不展示连接错误）

### 新增
- 需求页（DRD 列表）与 seed 数据（需求/蓝图/管道）
- 执行记录、契约页 seed 数据（参照 qtdata 模式）
- 组件测试覆盖：总览/传输/应用冒烟（6 个）

### 修复
- Web 平台崩溃：移除 dart:io Platform.environment（改用 dart-define 注入）
- 侧边栏 ListTile Material 祖先缺失断言
- 应用名统一为「量潮数据云」（manifest + index title）

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
