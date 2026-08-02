# 环境与版本工具（doctor.rs / version.rs）

本文档对应 `src/doctor.rs` 与 `src/version.rs`。

## doctor — 环境检查

检查本机 DataOps 环境：外部工具、数据目录、传输凭证。

```bash
qtcloud-data doctor                 # 检查；存在失败项时退出码 1
qtcloud-data doctor --no-fail       # 只输出报告，失败项不退出
qtcloud-data doctor --json          # 机器可读 JSON 报告（CI/Studio 用）
qtcloud-data doctor --fix-dirs      # 自动创建 .quanttide/data 目录结构
```

### 检查项（checks_with_dirs）

| 类别 | 内容 | required |
|------|------|----------|
| 工具 | `git` / `cargo` / `rustc` / `cue` | 前三 required，cue required（v0.2.2 计划降为 optional） |
| 工具（可选） | `python3` / `bash` | optional |
| 目录 | DRD / SPEC / BLUEPRINT / CONTRACT / PIPELINE / CATALOG | warn（缺省不阻断） |
| 凭证 | DROPBOX / BAIDU / GOOGLE / ONEDRIVE / SFTP / AWS | optional |

### 实现要点

- `data_dirs_with(lookup)`：env 查找函数注入（测试可替换，避免直接读写进程 env）
- 检查结果三态：`CheckStatus::Pass / Warn / Fail`
- 报告不含凭证值（只显示环境变量名）

## version — 规格版本管理

基于 `spec/` 与 `blueprint/` 目录的 git 历史：

```bash
qtcloud-data version list <name>     # git log --follow 版本历史
qtcloud-data version show <name> <version>   # 查看指定版本内容
qtcloud-data version diff <name> <v1> <v2>   # 比较两个版本
```

- 优先 `spec/`（`<name>-blueprint.cue`），回退旧 `blueprint/`（`<name>.cue`）
- 依赖 git 可用
