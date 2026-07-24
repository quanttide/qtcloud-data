# ROADMAP

> 当前版本 v0.1.0-alpha.1 | 目标版本 v0.1.0-beta.1：数据工程框架对齐
> Feature 愿景：CLI 命令对齐工程标准中定义的动词体系，让工程标准成为 CLI 和平台设计的单一事实源。

## v0.1.0 — Blueprint 五命令集（已完成）

Blueprint 命令已实现：`review / design / formalize / preview / version`，覆盖 Blueprint 三格式（.md / .cue / .html）生命周期。`cli/v0.1.0-alpha.1` 已发布。

---

## v0.1.0-beta.1 — 对齐升级版数据工程框架

核心目标：CLI 命令从"围绕 Blueprint"升级为"围绕工程标准动词体系"。

### 新框架背景

工程标准定义了完整的数据处理流程和对应的动词：

```
Context → clarify → Requirements (DRD) → design → Specification → implement → execute → report → transfer → Delivery
                                                      └─ Contract + Blueprint ─┘
```

CLI 不覆盖全流程——执行层（implement/execute）留给人来做。CLI 覆盖的重点是前期（clarify + design）和后端（report + transfer）。

### 命令结构升级

```
qtcloud-data
├── clarify                     ← NEW（v0.1.0-beta.1）
│   ├── from-chat <file>       从聊天记录/上下文生成 DRD
│   ├── list                   列出已有 DRD
│   └── show <name>            查看 DRD
├── design                      ← UPDATED（v0.1.0-beta.1）
│   ├── contract <drd>         从 DRD 生成 Contract（数据契约：输入输出规格）
│   ├── blueprint <drd>        从 DRD 生成 Blueprint（处理蓝图：工作流步骤）
│   ├── formalize <md>         md → CUE（保留，继承自 v0.1.0）
│   └── preview <cue>          CUE → HTML（保留）
├── review <file>              审计 DRD 或 Specification
├── contract {list, show}      不变
├── pipeline {list, show}      不变
├── catalog {}                 不变
├── process {}                 不变
├── transfer {send, receive, ls} 不变
└── version {list, show, diff} 不变
```

### 新旧概念对照

| 旧（v0.1.0） | 新（v0.1.0-beta.1） | 说明 |
|-------------|-------------|------|
| Blueprint（模糊） | Requirements + Specification | Blueprint 拆成两层 |
| `blueprint review` | `review`（提升为顶级命令） | 审计对象从 Blueprint 扩展到 DRD + Specification |
| `blueprint design new/edit` | `clarify from-chat` + `design contract/blueprint` | 旧 design 混合了需求和技术，新框架分两步 |
| `blueprint formalize` | `design formalize` | md→CUE 转换归到 design 子命令 |
| `blueprint preview` | `design preview` | 预览归到 design 子命令 |
| `blueprint version` | `version`（提升为顶级命令） | 版本管理独立出来 |
| — | `clarify` | 新增：从聊天记录/上下文生成 DRD |

### 对齐工程标准

目标：工程标准 (`docs/specification`) 中定义的每个动词，在 CLI 中都有对应命令。这样只要看工程标准就知道 CLI 和平台的定义对不对。

| 工程标准动词 | CLI 命令 | 状态 |
|-------------|---------|------|
| clarify | `clarify` | NEW |
| design | `design` | UPDATED |
| implement | — | 不覆盖 |
| execute | — | 不覆盖 |
| report | `report` | 后续版本 |
| transfer | `transfer` | 已有 |
| (审计) | `review` | 已有 |

### 实现顺序

1. `clarify from-chat` — 核心新命令，打通 context→DRD 流程
2. `design contract` / `design blueprint` — 从 DRD 生成 Specification
3. 老命令迁移：`blueprint *` → `design *` / `review` / `version`

---

## 版本目标

### v0.1.0-beta.1 — 框架对齐

**交付标准**：
- `clarify` 和 `design` 命令可用
- 老 `blueprint` 子命令迁移完成，向后兼容
- 命令结构与工程标准动词一一对应
- `cargo build && cargo test` 通过

**当前位置**：v0.1.0-alpha.1 → 冲向 v0.1.0-beta.1

### v0.5.0 — 好用

**交付标准**：
- 新人通过 CLI + 工程标准能快速上手历史项目
- `review` 能自动发现跨项目不一致模式
- 成为团队日常的数据蓝图治理工具
