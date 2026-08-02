# version（version.rs）

本文档对应 `src/version.rs`。

## 命令

基于 `spec/` 与 `blueprint/` 目录的 git 历史：

```bash
qtcloud-data version list <name>                 # git log --follow 版本历史
qtcloud-data version show <name> <version>      # 查看指定版本内容
qtcloud-data version diff <name> <v1> <v2>      # 比较两个版本
```

## 说明

- 优先 `spec/`（`<name>-blueprint.cue`），回退旧 `blueprint/`（`<name>.cue`）
- 依赖 git 可用
