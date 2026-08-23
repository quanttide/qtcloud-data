# pipeline（pipeline.rs）

本文档对应 `src/implementation/pipeline.rs`（实现资源）。

## 命令

| 命令 | 读取目录 | 说明 |
|------|---------|------|
| `pipeline list` | `PIPELINE_DIR` | 列出所有可用 pipeline |
| `pipeline show <name>` | `PIPELINE_DIR` | 查看 pipeline 定义详情 |

## 读取策略：文件直读为主，cue 可选增强

`pipeline list/show` 默认直接读取 `PIPELINE_DIR` 中的 `.yaml` / `.yml` / `.cue` / `.json` 文件：

- `list` 按文件名 stem 列出可用 pipeline。
- `show <name>` 按 `.yaml → .yml → .cue → .json` 顺序查找并输出原文件内容。

当目录不存在或需要读取 CUE 模块化目录时，命令保留 `cue export --out json` 兜底路径；因此 cue 是可选增强，不再是装完即用的硬依赖。
