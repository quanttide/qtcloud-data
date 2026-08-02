# qtcloud-data CLI — 开发者文档索引

本目录是 qtcloud-data CLI 的开发者文档入口。文档与 `src/` 模块**一一对应**（按模块组组织），
找模块 → 查映射表 → 读对应文档。

## 文档 ↔ 模块映射表

| 文档 | 对应模块 | 内容 |
|------|---------|------|
| [index.md](index.md)（本文件） | `main.rs`、`lib.rs`、`error.rs`、`registry.rs`、`util.rs` | 命令结构、文档映射、横切基础（错误模型 + 注册表/工具机制） |
| [transfer.md](transfer.md) | `transfer.rs`、`providers/` | 传输服务与 StorageProvider trait、添加新平台 |
| [catalog.md](catalog.md) | `catalog.rs` | 数据格式（registry/jobs/delivery-links 字段级定义） |
| [process.md](process.md) | `process.rs` | StepExecutor 编排（receive → pipeline → send） |
| [llm.md](llm.md) | `clarify.rs`、`design.rs`、`implement.rs`、`review.rs` | LLM 命令与 Handler 注入模式 |
| [specification.md](specification.md) | `spec.rs`、`blueprint_core.rs` | Specification YAML 契约与 Blueprint 工作流模型 |
| [blueprint.md](blueprint.md) | `blueprint.rs` | 蓝图定义查看 |
| [contract.md](contract.md) | `contract.rs` | 契约定义查看（文件直读） |
| [pipeline.md](pipeline.md) | `pipeline.rs` | 管道定义查看 |
| [doctor.md](doctor.md) | `doctor.rs` | 环境检查 |
| [version.md](version.md) | `version.rs` | 规格版本管理 |

贡献与发布流程见 [CONTRIBUTING.md](../CONTRIBUTING.md)。

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
  DRD        Spec       代码      编排      交付
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

## 横切基础（error.rs + registry.rs + util.rs）

横切基础是不属于任何命令模块、被所有模块共享的**机制层**：

### 错误模型（error.rs）

所有命令入口返回 `Result<(), CliError>`，`main` 顶层统一格式化 `错误: {err}` 并退出码 1。

```rust
// 命令实现（以 catalog 为例）
pub fn run(args: &CatalogArgs) -> Result<(), CliError> {
    match &args.action {
        CatalogAction::Show { name } => show(name),
        // ...
    }
}

fn show(name: &str) -> Result<(), CliError> {
    match registry.get(name) {
        Some(v) => { /* 打印 */ Ok(()) }
        None => Err(CliError::new(format!("未找到 volume: {name}"))),
    }
}
```

约定：

- 错误路径通过 `Result` 传播，**不直接 `std::process::exit(1)`**（仅 `main` 保留 bin 入口 exit）
- `Result<_, String>` 的公开函数已收敛为 `CliError`（`From<io::Error>/String/&str`）
- 错误路径因此可测试：`cmd_xxx(...).unwrap_err()`
- `CliError` 只携带用户可读消息（`Display` 即消息本体），不携带结构化错误码

### 注册表与工具机制（registry.rs + util.rs）

`registry.rs` 与 `util.rs` 提供 JSON 注册表与通用工具的**机制**（不定义数据内容，数据格式见 [catalog.md](catalog.md)）：

- `registry::Registry<T>`：JSON 注册表读写（open/get/entries/len/is_empty/insert/remove/save），
  **原子写盘**（临时文件 + rename，避免半写损坏）
- `util::catalog_dir()`：路径解析优先级 `CATALOG_DIR` > `DATA_ROOT/catalog` > `.quanttide/data/catalog`
- `util::now_utc()`：UTC 时间格式化（RFC 3339）

## 测试

测试分层与运行方式见 [CONTRIBUTING.md](../CONTRIBUTING.md)（测试分层节）。
覆盖率当前 83.7%，基线变更记录在 TODO 对应版本条目。

## 数据目录与命令的关系

命令的产物落在 `.quanttide/data/` 下（可用 `DATA_ROOT` 覆盖），字段级格式见 [catalog.md](catalog.md)：

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
- 想知道"某个模块怎么实现" → 查[映射表](#文档--模块映射表)
- 想知道"Specification 契约长什么样" → [specification.md](specification.md)
- 想知道"发布怎么走" → [CONTRIBUTING.md](../CONTRIBUTING.md)
