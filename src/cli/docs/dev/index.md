# qtcloud-data CLI — 开发者文档索引

本目录是 qtcloud-data CLI 的开发者文档入口。

## 文档索引

| 文档 | 内容 |
|------|------|
| [index.md](index.md)（本文件） | 命令结构总览、命令分类 |
| [transfer.md](transfer.md) | 架构概览、错误模型、LLM 注入、测试分层 |
| [specification.md](specification.md) | Specification YAML envelope 契约（CLI 与 Provider 共用） |
| [e2e-baseline.md](e2e-baseline.md) | 基线 e2e 验证记录与覆盖率基线 |

贡献与发布流程见 [CONTRIBUTING.md](../../CONTRIBUTING.md)。

## 命令结构总览

```
qtcloud-data
│
├── 生命周期命令（纵向流程，按数据流顺序）
│   ├── clarify     需求澄清：聊天记录 → DRD
│   ├── design      规格设计：DRD → Contract / Blueprint（contract / blueprint / formalize / preview）
│   ├── implement   代码实现：Blueprint → Python
│   ├── process     流程编排：receive → pipeline → send（StepExecutor）
│   └── transfer    数据传输：send / receive（6 平台）
│
├── 横向工具（全阶段可用，不隶属某个生命周期）
│   ├── doctor      环境检查：工具 / 目录 / 凭证（--no-fail / --json / --fix-dirs）
│   ├── review      质量审查：审计任意阶段产物（需求 / 设计 / 实现 / 交付）
│   ├── spec        Specification 工具：wrap（包装 envelope）/ validate（结构校验）
│   ├── catalog     数据目录：volume 登记（list / show / add / rm）
│   └── version     规格版本管理：git 历史（list / show / diff）
│
└── 查看命令（定义查看）
    ├── blueprint   蓝图定义查看（list / show）
    ├── contract    契约定义查看（list / show）
    └── pipeline    管道定义查看（list / show）
```

## 命令分类原则

命令按**用途定位**分三类，而非按名称平铺：

### 1. 生命周期命令（纵向）

数据交付的主流程，产物逐级流转：

```
clarify → design → implement → process → transfer
  DRD        Spec       代码       编排      交付
```

- 每个命令消费上一阶段的产物，产出一份新文档/代码/数据
- `design` 生成 Specification（Contract + Blueprint），`process` 按 blueprint 编排执行

### 2. 横向工具（横向）

不隶属某个生命周期，**随时可对任意阶段产物操作**：

- `doctor`：检查运行环境（工具、目录、凭证）——在任何阶段开始前可用
- `review`：审计任意阶段的产物质量——结构校验 + LLM 语义审查，输出 `review_decisions`
- `spec`：对 Specification 的包装/校验工具（与 review 同族，但只做确定性操作）
- `catalog`：数据目录登记，跨阶段共享的产物台账
- `version`：规格历史版本管理

> **review 的定位**：作为横向质量工具，它不构成独立生命周期——设计产物生成后可以自动触发、
> 流程执行前可以作为门禁，也可以随时手动审计任意阶段产物。与 `doctor` 对称：
> doctor 检查"环境是否就绪"，review 检查"产物是否合格"。

### 3. 查看命令

只读查看定义（需要数据目录中有对应文件；cue 为可选增强）。

## 数据目录与命令的关系

命令的产物落在 `.quanttide/data/` 下（可用 `DATA_ROOT` 覆盖）：

| 目录 | 产物 | 生产命令 | 消费命令 |
|------|------|---------|---------|
| `drd/` | 需求文档 .md | `clarify` | `design`、`review` |
| `spec/` | Specification .yaml/.md | `design` | `review`、`spec`、`implement` |
| `blueprint/` | Blueprint 定义 | `design blueprint` | `process`、`implement` |
| `contract/` | Contract 定义 | `design contract` | `review` |
| `pipeline/` | Pipeline 定义 | — | `process`、`pipeline list/show` |
| `catalog/` | registry / jobs / delivery-links | `catalog add`、`process`、`transfer` | `catalog list/show`、`review` |

## 快速导航

- 想知道"这个命令属于哪个环节" → 看[命令结构总览](#命令结构总览)
- 想知道"命令内部怎么实现" → [transfer.md](transfer.md)
- 想知道"Specification 契约长什么样" → [specification.md](specification.md)
- 想知道"发布怎么走" → [CONTRIBUTING.md](../../CONTRIBUTING.md)
