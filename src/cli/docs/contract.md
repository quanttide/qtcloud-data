# contract（contract.rs）

本文档对应 `src/contract.rs`。

## 命令

| 命令 | 读取目录 | 说明 |
|------|---------|------|
| `contract list` | `CONTRACT_DIR` | 列出所有可用契约 |
| `contract show <name>` | `CONTRACT_DIR` | 查看契约定义详情（名称不含扩展名） |

## 读取策略：文件直读为主（v0.2.1 落地）

`contract_names` 按扩展名（`.yaml` / `.yml` / `.cue` / `.json`）枚举目录文件，
`find_contract` 按 `.yaml → .yml → .cue → .json` 顺序查找。cue 为可选增强。

## 命名约定

- 契约名称不含扩展名（`contract show <name>`）
- 输出名称排序去重
