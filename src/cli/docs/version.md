# version（version.rs）

本文档对应 `src/spec/version.rs`（Specification 域）。

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

## 命令降级（v0.3 移除顶层）

顶层 `qtcloud-data version` 已废弃，主入口改为 `qtcloud-data spec version`：

```
qtcloud-data spec version list <name>
qtcloud-data spec version show <name> <version>
qtcloud-data spec version diff <name> <v1> <v2>
```

顶层 `version` 命令在 v0.3 移除前保留（帮助已标注废弃与替代入口）。
