# blueprint（blueprint.rs）

本文档对应 `src/spec/blueprint.rs`（Specification 域）。

## 命令

| 命令 | 读取目录 | 说明 |
|------|---------|------|
| `blueprint list` | `BLUEPRINT_DIR` | 列出所有可用 blueprint |
| `blueprint show <name>` | `BLUEPRINT_DIR` | 查看 blueprint 定义详情 |

## 读取策略：文件直读为主，cue 可选增强

`blueprint list/show` 默认直接读取 `BLUEPRINT_DIR` 中的 `.yaml` / `.yml` / `.cue` / `.json` 文件：

- `list` 按文件名 stem 列出可用 blueprint。
- `show <name>` 按 `.yaml → .yml → .cue → .json` 顺序查找并输出原文件内容。

当目录不存在或需要读取 CUE 模块化目录时，命令保留 `cue export --out json` 兜底路径；因此 cue 是可选增强，不再是装完即用的硬依赖。
