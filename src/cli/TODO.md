# TODO — CLI v0.1.0-beta.1 框架对齐

> 目标版本 v0.1.0-beta.1 | 测试覆盖率 ≥80%  
> 当前：v0.1.0-alpha.1 → v0.1.0-beta.1

---

## 交付边界（必读）

### 允许创建/修改的文件

```
src/clarify.rs             ← 新增：clarify 命令
src/design.rs              ← 新增：design 命令（contract/blueprint/formalize/preview）
src/review.rs              ← 新增：review 命令（从 blueprint/review 迁移 + 升级）
src/version.rs             ← 新增：version 命令（从 blueprint/version 迁移）
src/blueprint.rs           ← 修改：只剩 list/show，其他子命令迁移走
src/blueprint_core.rs      ← 修改：新增 clarify_prompt / design_contract_prompt / design_blueprint_prompt
src/lib.rs                 ← 修改：注册新模块
src/main.rs                ← 修改：重新定义 CLI 命令树
Cargo.toml                 ← 不修改（依赖不变）
tests/blueprint_test.rs    ← 追加 v0.1.0-beta.1 测试
tests/clarify_test.rs      ← 新增
tests/design_test.rs       ← 新增
```

### 禁止操作

- **禁止修改** transfer、process、pipeline、contract、catalog、providers 模块
- **禁止删除** blueprint.rs（list/show 保留）
- **禁止修改** 测试 fixture 文件

---

## 1. 新命令结构（对齐工程标准）

```
qtcloud-data
├── clarify                     ← NEW
│   └── from-chat <file>       从聊天记录/上下文生成 DRD (.md)
├── design                      ← REDESIGNED
│   ├── contract <drd>         从 DRD 生成 Contract (.cue + .md)
│   ├── blueprint <drd>        从 DRD 生成 Blueprint (.cue + .md + .html)
│   ├── formalize <md>         通用 md → CUE 转换
│   └── preview <cue>          通用 CUE → HTML 渲染
├── review <input>             从 blueprint 子命令提升为顶级
├── version {list,show,diff}   从 blueprint 子命令提升为顶级
├── blueprint {list,show}      保留，其他子命令迁移
├── contract {list,show}       不变
├── pipeline {list,show}       不变
├── catalog {}                 不变
├── process {}                 不变
└── transfer {send,receive,ls} 不变
```

---

## 2. `clarify` 命令

### 2.1 功能
- `clarify from-chat <file>` — 读聊天记录 .txt/.md，调 LLM 生成 DRD (.md) 到 `drd/` 目录

### 2.2 实现
- [ ] `src/clarify.rs` — 命令入口
  - [ ] `struct ClarifyArgs` — from-chat 子命令
  - [ ] `fn cmd_from_chat(input: &str, dir: &str)` — 读聊天记录 → LLM → 写 DRD
- [ ] `src/blueprint_core.rs` — 纯函数
  - [ ] `fn clarify_prompt(chat: &str) -> String` — 构造 clarify prompt
  - [ ] `fn drd_dir() -> String` — DRD 目录（`.quanttide/data/drd/`）
- [ ] 集成测试
  - [ ] `clarify from-chat` help 正常
  - [ ] `clarify_prompt` 包含 DRD 模板结构
  - [ ] `drd_dir` 返回正确默认值

---

## 3. `design` 命令（重新设计）

### 3.1 功能
- `design contract <drd>` — 读 DRD .md → LLM → Contract (.cue + .md)
- `design blueprint <drd>` — 读 DRD .md → LLM → Blueprint (.cue + .md + .html)
- `design formalize <md>` — 通用 md → CUE（保留）
- `design preview <cue>` — 通用 CUE → HTML（保留）

### 3.2 实现
- [ ] `src/design.rs` — 命令入口
  - [ ] `struct DesignArgs` — 子命令枚举
  - [ ] `fn cmd_contract(input: &str, dir: &str)` — DRD → Contract
  - [ ] `fn cmd_blueprint(input: &str, dir: &str)` — DRD → Blueprint
  - [ ] `fn cmd_formalize(input: &str, output: &Option<String>, dir: &str)` — md → CUE（保留）
  - [ ] `fn cmd_preview(input: &str, output: &Option<String>)` — CUE → HTML（保留）
- [ ] `src/blueprint_core.rs` — 纯函数
  - [ ] `fn design_contract_prompt(drd: &str) -> String` — Contract prompt
  - [ ] `fn design_blueprint_prompt(drd: &str) -> String` — Blueprint prompt
  - [ ] `fn design_formalize_prompt(md: &str) -> String` — 原有 formalize_prompt 重命名
  - [ ] `fn spec_dir() -> String` — 规格目录（`.quanttide/data/spec/`）
- [ ] 集成测试
  - [ ] 四个子命令 help 正常
  - [ ] prompt 包含对应的模板结构
  - [ ] `spec_dir` 返回正确默认值

---

## 4. 旧命令迁移

- [ ] `review` — 从 `blueprint review` 迁移为顶级 `review`
- [ ] `version` — 从 `blueprint version` 迁移为顶级 `version`
- [ ] `blueprint.rs` — 删除 review/design/formalize/preview/version 代码，只保留 list/show
- [ ] `main.rs` — 注册新顶级命令，移除旧 blueprint 子命令

---

## 5. 纯函数层更新

- [ ] `src/blueprint_core.rs`
  - [ ] 新增：`clarify_prompt`, `drd_dir`
  - [ ] 新增：`design_contract_prompt`, `design_blueprint_prompt`, `spec_dir`
  - [ ] 重命名：`formalize_prompt` → `design_formalize_prompt`
  - [ ] 保留：`review_prompt`, `extract_cue`, `render_html`, `design_template`, `to_camel`, `blueprint_dir`, `resolve_*`
  - [ ] 删除：无

---

## 6. Build & CI

- [ ] `cargo build` 通过
- [ ] `cargo test` 全量通过
- [ ] `cargo clippy` 无 warning
- [ ] `cargo fmt` 检查通过
- [ ] 测试覆盖率 ≥80%（纯逻辑层）
