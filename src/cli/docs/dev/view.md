# 查看命令（blueprint.rs / contract.rs / pipeline.rs）

本文档对应 `src/blueprint.rs`、`src/contract.rs`、`src/pipeline.rs`。

## 命令一览

| 模块 | 命令 | 读取目录 | 说明 |
|------|------|---------|------|
| `blueprint.rs` | `blueprint list/show` | `BLUEPRINT_DIR` | 蓝图定义查看 |
| `contract.rs` | `contract list/show` | `CONTRACT_DIR` | 契约定义查看 |
| `pipeline.rs` | `pipeline list/show` | `PIPELINE_DIR` | 管道定义查看 |

## 读取策略：文件直读为主，cue 为可选增强

查看命令的核心问题是"目录里的定义文件怎么列出来/读出来"。分两种策略：

| 模块 | 现状 | 策略 |
|------|------|------|
| `contract.rs` | ✅ **文件直读为主**（v0.2.1 落地） | `contract_names` 按扩展名（`.yaml/.yml/.cue/.json`）枚举 + `find_contract` 按序查找；cue 为可选增强 |
| `pipeline.rs` | 🟡 cue 解析 | v0.2.2 计划改为文件直读（对齐 contract.rs） |
| `blueprint.rs` | 🟡 cue 解析 | v0.2.2 计划改为文件直读（对齐 contract.rs） |

**用户体验动机**（v0.2.2）：cue 对非 CUE 格式目录要求 `cue.mod/module.cue`，且是外部工具依赖——
用户装完 CLI 应能直接 list/show，不应暴露 cue 模块概念。文件直读后 cue 从"必需依赖"降为"可选工具"。

## 命名约定

- 契约文件支持多种扩展名，`find_contract` 按 `.yaml → .yml → .cue → .json` 顺序查找
- 契约名称不含扩展名（`contract show <name>`）
- 输出名称排序去重
