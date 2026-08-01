---
name: qtcloud-devops
description: 使用 qtcloud-devops CLI 执行本项目的 DevOps 流程：子模块管理、构建、测试、发布、规划审计。当用户要求提交、发布、检查状态、审计代码、或执行标准研发流程时使用。
---

# qtcloud-devops — 量潮 DevOps 工作流程

## 概述

`qtcloud-devops` 是量潮数据工程的 DevOps 命令行工具，覆盖从代码到发布的完整生命周期。

**核心流程：** `code → build → test → release`

**当前版本：** v0.11.0（安装于 `~/.cargo/bin/qtcloud-devops`，源码在 `domains/quanttide-devops/apps/qtcloud-devops`）

## 全局命令

| 命令 | 用途 |
|---|---|
| `qtcloud-devops status` | 聚合查看 build / test / release / contract / plan 状态 |
| `qtcloud-devops audit` | 聚合执行 build / test / release 审计 |
| `qtcloud-devops doctor status` | 检查系统依赖命令状态（git、gh、python、rust 等） |

## Stage 说明

### 1. code — 代码管理

在提交代码前运行审计，确保代码质量：

```sh
qtcloud-devops code status [PATH]    # 查看子模块同步状态（--offline 可离线检查）
qtcloud-devops code audit [PATH]     # 审计：scope 目录、TODO/FIXME 密度、语法检查（--json 输出供 plan 消费）
```

### 2. build — 构建

```sh
qtcloud-devops build status          # 查看构建状态
qtcloud-devops build clean           # 清理构建产物（target/、dist/ 等）
qtcloud-devops build audit           # 审计：编译器配置、CI 工作流、依赖声明
```

### 3. test — 测试

```sh
qtcloud-devops test status           # 查看测试状态
qtcloud-devops test clean            # 清理测试产物（覆盖率报告等）
qtcloud-devops test audit            # 审计：测试覆盖率、错误变体覆盖、门禁达标
```

### 4. release — 发布

发布前必须完成 build 和 test 审计。发布流程：

```sh
qtcloud-devops release status                              # 查看当前版本状态（版本号、标签、CHANGELOG）
qtcloud-devops release audit [-v vX.Y.Z] [--scope <scope>] # 发布预检：版本号、CHANGELOG、标签冲突、远程可达性
qtcloud-devops release publish [-v vX.Y.Z] [-y] [-f] [--dry-run] [--registry <target>]
```

`release publish` 参数说明：

| 参数 | 用途 |
|---|---|
| `-v, --version <VERSION>` | 版本号，格式 `vX.Y.Z` 或 `scope/vX.Y.Z`（如 `cli/v0.5.0`）；省略时自动检测 |
| `-y, --yes` | 跳过用户确认 |
| `-f, --force` | 强制重新发布：删除已存在的 tag 和 Release 后重新创建 |
| `--dry-run` | 仅预览，不执行任何操作 |
| `--registry <REGISTRY>` | CI 发布目标（`py-pi` / `pub-dev` / `crates`），仅打印提示不执行发布 |

## 跨 Stage 命令

### plan — 规划管理（ROADMAP.md / TODO.md）

| 命令 | 用途 |
|---|---|
| `qtcloud-devops plan status [--scope <scope>]` | 查看 scope 规划进度（省略 scope 时自动检测当前目录所属 scope） |
| `qtcloud-devops plan audit` | 审计 ROADMAP 与 TODO 的关系：ROADMAP 是完整规划，TODO 是待办 |
| `qtcloud-devops plan doctor [--scope <scope>]` | 修复 ROADMAP 和 TODO 的格式：标准化版本头、分类、checkbox |
| `qtcloud-devops plan clean [--scope <scope>]` | 删除 scope 已完成条目 |
| `qtcloud-devops plan todo-from-audit [--scope <scope>]` | 从审计 JSON 更新 TODO.md（stdin 读 JSON，配合 `code audit --json`） |
| `qtcloud-devops plan roadmap-from-audit [--scope <scope>]` | 从审计 JSON 更新 ROADMAP.md（stdin 读 JSON） |

### contract — 契约状态

```sh
qtcloud-devops contract status       # 查看契约配置与状态
```

## 推荐工作流

### 日常开发

```sh
# 1. 检查环境
qtcloud-devops doctor status

# 2. 代码审计
qtcloud-devops code audit

# 3. 构建验证
qtcloud-devops build status

# 4. 测试
qtcloud-devops test status
```

### 规划维护

```sh
# 1. 审计 ROADMAP 与 TODO 一致性
qtcloud-devops plan audit

# 2. 修复格式问题
qtcloud-devops plan doctor

# 3. 用 code audit JSON 回填 TODO/ROADMAP
qtcloud-devops code audit --json | qtcloud-devops plan todo-from-audit
```

### 发布新版本

```sh
# 1. 预检
qtcloud-devops release status

# 2. 发布
qtcloud-devops release publish -v v0.2.0 -y

# 3. 验证
qtcloud-devops release status
```

## 注意事项

- 发布前建议运行 `release status` 预检版本状态；`publish` 内部自带校验，不强制要求 `release audit`
- `code audit` 应在每次提交前运行，确保子模块同步和代码质量
- `plan doctor` 可自动修复 ROADMAP.md 和 TODO.md 的格式问题
- `plan todo-from-audit` / `roadmap-from-audit` 依赖 `code audit --json` 的输出，通过 stdin 管道传入
- 子模块路径变更后，请运行 `qtcloud-devops code status` 确认同步状态
- scope 相关命令（plan/release audit）都支持 `--scope` 参数或自动检测当前目录所属 scope
