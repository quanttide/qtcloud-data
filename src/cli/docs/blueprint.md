# blueprint（blueprint.rs）

本文档对应 `src/spec/blueprint.rs`（Specification 域）。

## 命令

| 命令 | 读取目录 | 说明 |
|------|---------|------|
| `blueprint list` | `BLUEPRINT_DIR` | 列出所有可用 blueprint |
| `blueprint show <name>` | `BLUEPRINT_DIR` | 查看 blueprint 定义详情 |

## 读取策略：cue 解析（v0.2.2 计划改文件直读）

当前 `blueprint list/show` 通过 `cue export --out json` 解析（需要 cue CLI 与目录模块化）。

**v0.2.2 计划**：改为文件直读为主（对齐 `contract.rs`），cue 降为可选增强——
cue 对非 CUE 格式目录要求 `cue.mod/module.cue`，用户装完 CLI 应能直接 list/show，
不应暴露 cue 模块概念。
