# 编排（process.rs）

本文档对应 `src/process.rs`。

## 职责

`process` 命令按 blueprint 编排完整流程：**receive → pipeline → send**。
把数据交付的碎片环节串成一条可重复命令：

```bash
qtcloud-data process <customer-id> <source-url> --blueprint <name>
```

## StepExecutor 状态机

v0.2.1 起由 `StepExecutor` 状态机驱动三步，**库内组合**（不再自我 re-exec）：

```
Step 1  receive  → transfer::receive（QTDATA_CLI env 时委派外部 CLI）
Step 2  pipeline → run_pipeline（按扩展名分发执行脚本）
Step 3  send     → transfer::send（进程内）
```

- 统一失败出口：任一步失败 → `fail()` 写 failed job 记录 + 日志行 → 返回 `CliError`
- 每步把完成日志写入 `log_lines`，最终落到 `CATALOG_DIR/jobs/<job-id>.log`

## Pipeline 执行（run_pipeline）

按扩展名分发：

| 扩展名 | 执行器 |
|--------|--------|
| `.py` | `python3` |
| `.sh` | `bash` |
| 其他 | 直接 exec（可执行文件） |

pipeline 单步输出为 `final.csv`。

## 产物与记录

- **job 记录**：成功或失败都写入 `CATALOG_DIR/jobs.json`（字段见 [catalog.md](catalog.md)），
  `source_url` 脱敏（redact query/fragment，不含 token）
- **日志**：`CATALOG_DIR/jobs/<job-id>.log`
- **交付登记**：成功交付后把最终产物登记到 `CATALOG_DIR/registry.json`（provider=`process`，
  source=`process:<job-id>`，status=`delivered`）；登记失败只输出 warning，不反转交付结果
- **交付链接**：`send` 生成链接写入 `CATALOG_DIR/delivery-links.json` 与 `--output` 链接文件

## 关键环境变量

| 变量 | 默认值 | 用途 |
|------|--------|------|
| `PIPELINE` | `csv-standard` | 默认 pipeline 名称 |
| `PIPELINE_DIR` | `.quanttide/data/pipeline` | 管道定义目录 |
| `WORKDIR` | 系统临时目录下的 `qtcloud-data` | 流程执行工作目录 |
| `QTDATA_CLI` | `qtcloud-data` | 自身命令路径（transfer 委派逃生舱） |
| `CATALOG_DIR` | `.quanttide/data/catalog` | 记录与登记目录 |
