# doctor（doctor.rs）

本文档对应 `src/doctor.rs`。

## 命令

检查本机 DataOps 环境：外部工具、数据目录、传输凭证。

```bash
qtcloud-data doctor                 # 检查；存在失败项时退出码 1
qtcloud-data doctor --no-fail       # 只输出报告，失败项不退出
qtcloud-data doctor --json          # 机器可读 JSON 报告（CI/Studio 用）
qtcloud-data doctor --fix-dirs      # 自动创建 .quanttide/data 目录结构
```

## 检查项（checks_with_dirs）

| 类别 | 内容 | required |
|------|------|----------|
| 工具 | `git` / `cargo` / `rustc` | required |
| 工具（可选） | `python3` / `bash` / `cue` | optional；cue 仅用于 CUE 模块化目录查看增强 |
| 目录 | DRD / SPEC / BLUEPRINT / CONTRACT / PIPELINE / CATALOG | warn（缺省不阻断） |
| 凭证 | DROPBOX / BAIDU / GOOGLE / ONEDRIVE / SFTP / AWS | optional |

## 实现要点

- `data_dirs_with(lookup)`：env 查找函数注入（测试可替换，避免直接读写进程 env）
- 检查结果三态：`CheckStatus::Pass / Warn / Fail`
- 报告不含凭证值（只显示环境变量名）
