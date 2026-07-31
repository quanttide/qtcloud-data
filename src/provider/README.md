# `qtcloud-provider`

`qtcloud-provider` 是 Data Cloud 的 Go 后端服务。当前发布版 `v0.0.1` 仍是骨架；本地 `[Unreleased]` 已开始承接 CLI 生成的 Specification/Blueprint YAML，并提供最小可运行的 Pipeline API，供 Studio 浏览和演示。

## 当前能力

- 读取 CLI 生成的 legacy Blueprint YAML 和 Specification envelope YAML。
- `GET /blueprints`：返回 Blueprint 列表。
- `GET /blueprints/{name}`：返回单个 Blueprint 详情。
- `POST /blueprints/{name}/runs`：执行一次带 `resource` 的 Blueprint pipeline。
- `GET /process/jobs`：返回 Provider 侧执行记录。
- `GET /process/jobs/{id}`：返回单条执行记录、输入输出和 step 详情。

## Pipeline 模型

Provider 优先读取 `pipeline.start_at` / `pipeline.states`，并兼容旧的 `pipeline.steps` 线性步骤。这个模型参考 AWS Step Functions / Amazon States Language 的核心结构，但只是内部简化 YAML DSL，不是完整 ASL 实现。

当前支持的最小状态机字段：

- `start_at`：起始 state。
- `states`：state map。
- `type: task`：当前唯一可执行 state 类型。
- `resource`：执行资源，如 `builtin:copy`、`python:<script>`、`bash:<script>`。
- `next` / `end`：顺序转移或终止。

当前暂不实现 `Choice`、`Parallel`、`Map`、`Retry`、`Catch`、`InputPath`、`ResultPath`、`OutputPath`、AWS ARN/IAM 等完整 Step Functions 能力。

## 执行资源

- `builtin:copy`：把当前输入文件复制到当前步骤输出，适合 smoke test 和内部演示。
- `python:<script>`：用 `PYTHON_BIN` 或系统默认 Python 运行脚本，参数为 `<input> <output>`。
- `bash:<script>`：用 `bash` 运行脚本，参数为 `<input> <output>`。

Pipeline 执行时，上一步输出文件会作为下一步输入文件。

## 持久化与路径边界

job store 默认按以下优先级选择路径：

1. `JOB_STORE_PATH`
2. `CATALOG_DIR/provider-jobs.json`
3. `DATA_ROOT/catalog/provider-jobs.json`
4. `.quanttide/data/catalog/provider-jobs.json`

可用以下环境变量收紧本地执行边界：

- `SPEC_DIR`：Blueprint/Specification YAML 目录。
- `DATA_ROOT`：默认数据根目录。
- `PIPELINE_INPUT_DIR`：设置后，run API 的 `input_path` 必须位于该目录下。
- `PIPELINE_WORK_ROOT`：设置后，run API 的 `work_dir` 必须位于该目录下；未传 `work_dir` 时按 job id 在该目录下创建。该根目录不存在时 Provider 会自动创建。
- `PIPELINE_SCRIPT_DIR`：脚本资源必须位于该目录下；Provider 会解析真实路径，拒绝 symlink/junction 逃逸。

## 本地运行

从仓库根目录进入 Provider：

```powershell
cd src/provider
$repoRoot = (Resolve-Path ..\..).Path
$env:PORT = "8080"
$env:SPEC_DIR = Join-Path $repoRoot "src\cli\.quanttide\data\spec"
$env:DATA_ROOT = Join-Path $repoRoot "src\cli\.quanttide\data"
$env:PIPELINE_WORK_ROOT = Join-Path $repoRoot ".codex-run\provider-work"
go run .\cmd\qtcloud-provider
```

运行一次 Blueprint：

```powershell
$repoRoot = (Resolve-Path ..\..).Path
$inputPath = Join-Path $repoRoot ".codex-run\demo-raw.csv"
Invoke-RestMethod -Method Post `
  -Uri "http://127.0.0.1:8080/blueprints/customer-chat/runs" `
  -ContentType "application/json" `
  -Body (@{
    customer_id = "demo"
    input_path = $inputPath
  } | ConvertTo-Json)
```

## 验证

```powershell
go test ./...
go vet ./...
```
