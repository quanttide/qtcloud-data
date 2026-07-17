# TODO — CLI Blueprint 五命令集

> 目标版本 v0.1.0 | 测试覆盖率 ≥80%  
> 依赖：`quanttide-data-core`（toolkit/packages/rust）发布后集成  
> LLM：AI 读 `quanttide-agent-toolkit` 源码后接入

---

## 交付边界（必读）

### 允许创建/修改的文件

```
src/blueprint/review.rs      ← 新增
src/blueprint/design.rs      ← 新增（本期只做模板生成）
src/blueprint/formalize.rs   ← 新增
src/blueprint/preview.rs     ← 新增
src/blueprint/version.rs     ← 新增
src/blueprint/mod.rs         ← 修改（注册子命令）
src/main.rs                  ← 不修改（已有 Blueprint 注册）
Cargo.toml                   ← 修改（加新依赖，如有需要）
tests/integration_test.rs    ← 追加测试
docs/user/blueprint.md       ← 新增
docs/dev/blueprint.md        ← 新增
```

### 禁止操作

- **禁止修改** src/transfer.rs、src/process.rs、src/pipeline.rs、src/contract.rs、src/catalog.rs、src/lib.rs
- **禁止修改** 任何测试 fixture 文件
- **禁止在** src/ 下创建非 blueprint 命名的文件
- **禁止修改** README.md、CHANGELOG.md（交付时统一更新）

### 交付验证

每完成一个 checkbox，运行 `cargo build && cargo test && cargo clippy && cargo fmt --check` 确认不破坏已有功能。

---

## 1. LLM 接入准备

- [ ] 读 `quanttide-agent-toolkit` 源码，理解 agent 初始化、prompt 构造、响应解析
- [ ] 确认 `LLM_API_KEY` 环境变量配置方式

---

## 2. `review` 命令 — 审计已有 Blueprint

### 2.1 模块实现

- [ ] `src/blueprint/review.rs` — 审计已有 Blueprint，输出问题清单
  - [ ] `fn read_blueprint(path: &Path) -> Result<Blueprint>` — 读取 .cue 文件（通过 toolkit 解析）
  - [ ] `fn build_review_prompt(blueprint: &Blueprint) -> String` — 构造审计 prompt
  - [ ] `fn run_review(blueprint: &Blueprint, agent: &Agent) -> Result<ReviewReport>` — 调 LLM 审计
  - [ ] `struct ReviewReport` — 问题清单：严重程度、位置、建议修复
- [ ] `src/blueprint/mod.rs` — 注册 `review` 子命令
  - [ ] 参数：`--input`（.cue 路径或 Blueprint 名称）

### 2.2 集成测试

- [ ] 输入 ghtorrent/blueprint.cue → 输出包含结构化问题清单
- [ ] 验证 ReviewReport 包含严重程度、位置、建议
- [ ] 验证空 Blueprint 或缺失字段时能正确报出问题

---

## 3. `formalize` 命令 — Markdown → CUE

### 3.1 模块实现

- [ ] `src/blueprint/formalize.rs` — 读取 .md 文件，调 LLM 形式化为 CUE
  - [ ] `fn read_markdown(path: &Path) -> Result<String>` — 读取 .md 文件内容
  - [ ] `fn build_prompt(markdown: &str) -> String` — 构造 LLM prompt
  - [ ] `fn call_llm(prompt: &str, agent: &Agent) -> Result<String>` — 调 LLM，返回 CUE 代码
  - [ ] `fn write_cue(cue_code: &str, output_path: &Path) -> Result<()>` — 写入 .cue 到 `BLUEPRINT_DIR`
  - [ ] `fn validate_cue(cue_path: &Path) -> Result<()>` — 调 `cue vet` 验证输出
- [ ] `src/blueprint/mod.rs` — 注册 `formalize` 子命令
  - [ ] 参数：`--input`（.md 路径）、`--output`（可选，默认同名 .cue）

### 3.2 集成测试

- [ ] 用 `data/profile/ghtorrent/blueprint.md` 作为输入
- [ ] 验证输出 .cue 通过 `cue vet`
- [ ] 验证输出 .cue 包含 `#Blueprint` 实例
- [ ] 验证错误输入（空文件、非 Blueprint 内容）能报错而非崩溃

---

## 4. `design` 命令 — 创建/编辑 Markdown Blueprint

- [ ] `src/blueprint/design.rs`
  - [ ] `fn generate_template(name: &str) -> String` — 根据 `#Blueprint` 类型生成 .md 模板
  - [ ] `blueprint design --new <name>` — 创建新 Blueprint .md 文件
  - [ ] `blueprint design --edit <name>` — 打开已有 .md 文件（调用 $EDITOR）
- [ ] 集成测试：生成模板非空、包含必要章节标题

---

## 5. `preview` 命令 — CUE → HTML

- [ ] `src/blueprint/preview.rs`
  - [ ] `fn cue_to_html(cue_path: &Path, output_path: &Path) -> Result<()>` — 渲染为 HTML
  - [ ] 参考 `data/profile/ghtorrent/blueprint.html` 的样式
  - [ ] 支持版本并排对比（多个 CUE 文件同时输入）
- [ ] 集成测试：输入 .cue 输出非空 .html

---

## 6. `version` 命令 — 版本管理

- [ ] `src/blueprint/version.rs`
  - [ ] `blueprint version list` — 列出版本历史
  - [ ] `blueprint version show <ver>` — 查看指定版本
  - [ ] `blueprint version diff <v1> <v2>` — 版本差异
- [ ] 集成测试：两版本 diff 输出包含新增/删除/变更

---

## 7. 现有命令增强

- [ ] `blueprint list` — 增加 `--format json` 输出
- [ ] `blueprint show` — 增加 `--format yaml|json` 选项

---

## 8. Build & CI

- [ ] `cargo build` 通过
- [ ] `cargo test` 全量通过
- [ ] `cargo clippy` 无 warning
- [ ] `cargo fmt` 检查通过
- [ ] CI workflow：`rust-build.yml` 补充 lint + test 步骤
- [ ] 测试覆盖率 ≥80%（`cargo tarpaulin`）

---

## 覆盖率目标：≥80%
