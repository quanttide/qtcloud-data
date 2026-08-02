# LLM 命令（clarify.rs / design.rs / implement.rs / review.rs）

本文档对应 `src/clarify.rs`、`src/design.rs`、`src/implement.rs`、`src/review.rs`。

## 命令与职责

| 模块 | 命令 | 输入 → 输出 | 产物 |
|------|------|------------|------|
| `clarify.rs` | `clarify from-chat` | 聊天记录 → DRD | `drd/<stem>.md` |
| `design.rs` | `design contract` | DRD → Contract | `spec/<stem>-contract.{yaml,md}` |
| `design.rs` | `design blueprint` | DRD → Blueprint | `spec/<stem>-blueprint.{yaml,md,html}` |
| `design.rs` | `design formalize` | Markdown → YAML | 指定或 `spec/<stem>.yaml` |
| `design.rs` | `design preview` | Blueprint YAML → HTML | 指定或 `<stem>.html`（无 LLM） |
| `implement.rs` | `implement` | Blueprint → Python | 指定或 `<stem>.py` |
| `review.rs` | `review` | 任意阶段产物 → 审计报告 | 打印（横向工具） |

## Handler 构造器注入（v0.2.1）

每个 LLM 命令使用 **Handler struct + 构造器注入**：

```rust
pub struct DesignHandler {
    llm: quanttide_agent::LLM,
}

impl DesignHandler {
    pub fn new(llm: quanttide_agent::LLM) -> Self { Self { llm } }
    pub fn run(&self, args: &DesignArgs) -> Result<(), CliError> { /* ... */ }
}
```

- **生产路径**：`main.rs` 里 `DesignHandler::new(quanttide_agent::LLM::default())`
  （`LLM::default()` 从 `Settings::from_env()` 读模型/base_url/api_key）
- **测试路径**：复用 quanttide-agent 的 `HttpClient` 抽象，`lib.rs test_support::fake_llm(content)`
  构造假 LLM（`LLM::with_client` 注入 FakeHttpClient），**不发起真实网络请求**

```rust
// 测试示例（src/design.rs）
let handler = DesignHandler::new(fake_llm("## 输入契约\n| 字段名 | 类型 | ..."));
handler.run(&DesignArgs { action: DesignAction::Contract { input } }).unwrap();
```

## LLM 调用约定

- 每个命令在**自己模块内**构建 prompt（`clarify_prompt` / `design_*_prompt` / `review_prompt` / `implement_*_prompt`，v0.2.2 起就近回迁）→ `Message::new("user", prompt)` →
  `llm.complete(&messages, CompleteOptions::default())`
- 响应解析：design 解析 LLM 输出的 Markdown 表格（`contract_tables_to_yaml` /
  `blueprint_table_to_yaml`）；formalize / implement 提取代码块（`extract_cue` / `extract_python_fn`）
- 错误处理：LLM 调用失败返回 `Err(CliError)`；review 例外——LLM 失败时降级输出
  结构校验结果，不阻塞（确定性校验本身有价值）

## review 的定位（横向工具）

`review` 审计任意阶段产物（需求 / 设计 / 实现 / 交付），分两层：

1. **确定性层**：`quanttide_data::validate(&blueprint)` 结构校验（纯代码，无网络）
2. **语义层**：`review_prompt` + LLM 审查（带已知结构问题做语义级判断，按
   【严重】/【警告】/【建议】输出）

LLM 失败不阻塞——降级输出结构校验问题。未来输出结构化 `review_decisions`
（v0.2.2 manifest 契约的一部分）供下游消费。
