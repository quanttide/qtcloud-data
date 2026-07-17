# ROADMAP

> 当前版本 v0.0.5 | 目标版本 v0.1.0：Blueprint 完整生命周期  
> Feature 愿景：**数据蓝图治理工具** — 把不同时期、不同版本的 Blueprint 规范到统一格式，让新人通过工具快速吸收历史项目经验。v0.5 以后完全实现。

## v0.1.0 — Blueprint 五命令集

核心目标：让 CLI 能够操作 Blueprint 的三格式（.md / .cue / .html）完整生命周期，覆盖新项目创建和老项目整理两条路径。

### 当前状态

Blueprint 命令已有 `list` 和 `show` 两个子命令，底层通过调用 `cue` CLI 实现。

### 工作流

```
review → design → formalize → preview → version
  ↑                                                    │
  └──────────── 循环迭代，每次 review 更少问题 ──────────┘
```

| 命令 | 输入 | 输出 | 功能 |
|------|------|------|------|
| `review` | 已有 Blueprint | 问题清单 | 审计已有 Blueprint，找缺口和不一致。老项目进入系统的入口 |
| `design` | — | `.md` | 创建/编辑人类可读的 Blueprint 文档 |
| `formalize` | `.md` | `.cue` | 将 Markdown 形式化为 CUE 结构化定义 |
| `preview` | `.cue` | `.html` | 从 CUE 生成可视化页面 |
| `version` | — | 版本元数据 | 维护版本历史、show、diff |

### review 与 design 的分工

- `review` 面向**已有项目**：审计现有 Blueprint，输出"哪里不严谨、哪里缺信息"
- `design` 面向**缺口**：review 发现问题后，design 补充缺失部分；新项目则直接 design

### 实现顺序

1. `formalize` — 核心命令，打通 md→cue 流程
2. `review` — 审计命令，老项目入口
3. `design` — 模板生成与编辑
4. `preview` — cue→html 渲染
5. `version` — 版本管理

### 依赖

- `packages/quanttide-data-toolkit/packages/rust/` — Blueprint 数据模型
- `quanttide-agent-toolkit` — LLM 调用（AI 读源码理解接口）

### LLM 接入

CLI 通过 `quanttide-agent` 统一接口调用 LLM，AI 直接读 `quanttide-agent-toolkit` 源码理解调用方式。环境变量 `LLM_API_KEY` 配置 API key。
