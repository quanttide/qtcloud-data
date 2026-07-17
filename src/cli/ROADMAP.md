# ROADMAP

> 当前版本 v0.0.5 | 目标版本 v0.1.0：Blueprint 完整生命周期

## v0.1.0 — Blueprint 命令集

核心目标：让 CLI 能够操作 Blueprint 的三格式（.md / .cue / .html）完整生命周期。

### 当前状态

Blueprint 命令已有 `list` 和 `show` 两个子命令，底层通过调用 `cue` CLI 实现。路径解析通过 `BLUEPRINT_DIR` 环境变量（默认 `.quanttide/data/blueprint/`）。

### 新增子命令

| 命令 | 输入 | 输出 | 功能 |
|------|------|------|------|
| `formalize` | `.md` | `.cue` | 将 Markdown 蓝图文档形式化为 CUE 结构化定义 |
| `design` | — | `.md` | 创建/编辑人类可读的 Blueprint 文档 |
| `preview` | `.cue` | `.html` | 从 CUE 生成可视化 HTML 页面 |
| `version` | — | 版本元数据 | 维护版本历史与变更记录 |

### `formalize` 实现要点

- 读取 `.md` Blueprint 文档，提取结构化信息
- 调用 LLM（`LLM_API_KEY`）将 Markdown 内容映射到 `#Blueprint` CUE 类型
- 输出 `.cue` 文件，写入 `BLUEPRINT_DIR`
- 输出文件可通过 `blueprint show` 验证

### 依赖

- `packages/quanttide-data-toolkit/packages/rust/` — Blueprint 数据模型（`#Blueprint`, `#Pipeline`, `#Step`, `#Contract` 类型定义）

### 实现顺序

1. `formalize` — 核心命令，打通 md→cue 流程
2. `design` — 辅助命令，提供模板和编辑引导
3. `preview` — 渲染命令，cue→html
4. `version` — 版本管理命令

---

## 待讨论

- `design`/`preview`/`version` 的具体交互模式需等 `formalize` 验证通过后再细化
- `formalize` 是调用外部 LLM API 还是依赖本地 `cue` 工具？初步方案是 LLM API
