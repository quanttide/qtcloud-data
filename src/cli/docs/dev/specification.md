# Specification YAML 契约

`qtcloud-data spec wrap` 把已有 Blueprint YAML 包装成稳定 envelope，供 CLI、Provider 和 Studio 共用。

Blueprint 是设计和实现之间的中间规格，不是执行代码。它借鉴 AWS Step Functions / Amazon States Language 的状态机表达：用结构化规格描述工作流入口、状态、转移和结束条件，再由 `implement` 转成 Python 等代码，或由 Provider/Pipeline 执行器解释执行。这样可以把“业务设计”与“代码实现”分离，降低人和 AI 的认知负担。

参考边界：AWS Step Functions 官方的 Amazon States Language 是 JSON-based 状态机定义，核心字段包括 `StartAt`、`States`、`Task`、`Resource`、`Next` 和 `End`。本项目的 Blueprint 是参考这些概念的内部 YAML DSL：v0.2.0 只实现线性 `task` 链和文件路径式输入输出传递，不宣称兼容完整 ASL。官方参考：

- [Getting started with AWS Step Functions](https://docs.aws.amazon.com/step-functions/latest/dg/getting-started.html)
- [State machine structure](https://docs.aws.amazon.com/step-functions/latest/dg/statemachine-structure.html)
- [Task workflow state](https://docs.aws.amazon.com/step-functions/latest/dg/state-task.html)
- [Amazon States Language](https://docs.aws.amazon.com/step-functions/latest/dg/concepts-amazon-states-language.html)

## Specification Envelope

```yaml
api_version: qtcloud.quanttide.com/v1alpha1
kind: Specification
metadata:
  name: sample
  generated_by: qtcloud-data-cli
  source_path: .quanttide/data/spec/sample-blueprint.yaml
spec:
  blueprint:
    name: sample
    contract:
      input:
        schema: "raw: string"
        format: CSV
      output:
        schema: "clean: string"
        format: CSV
    pipeline:
      name: sample-pipeline
      start_at: clean
      states:
        clean:
          type: task
          from: raw
          to: clean
          desc: trim whitespace
          resource: builtin:copy
          end: true
      steps:
        - name: clean
          from: raw
          to: clean
          desc: trim whitespace
          resource: builtin:copy
    status: draft
    created_at: "2026-07-30T00:00:00+00:00"
    updated_at: "2026-07-30T00:00:00+00:00"
```

CLI 读取端兼容两种格式：

- 旧格式：文件顶层就是 Blueprint。
- 新格式：文件顶层是 Specification envelope，Blueprint 位于 `spec.blueprint`。

```bash
qtcloud-data spec wrap .quanttide/data/spec/sample-blueprint.yaml
qtcloud-data spec validate .quanttide/data/spec/sample-spec.yaml
```

Provider 对齐时优先读取 envelope 中的 `api_version`、`kind` 和 `spec.blueprint.pipeline`。

## Blueprint 工作流模型

`pipeline.steps` 是旧版本兼容字段，便于当前 `implement` 继续按线性步骤生成代码。`pipeline.start_at` 和 `pipeline.states` 是新的状态机字段，供 Provider/Pipeline 执行器和 Studio 更清楚地理解流程。

```yaml
pipeline:
  name: customer-chat-pipeline
  start_at: load
  states:
    load:
      type: task
      from: raw CSV
      to: validated CSV
      desc: check required fields and date format
      resource: builtin:copy
      next: standardize
    standardize:
      type: task
      from: validated CSV
      to: final CSV
      desc: normalize amount and date fields
      resource: python:standardize.py
      end: true
  steps:
    - name: load
      from: raw CSV
      to: validated CSV
      desc: check required fields and date format
      resource: builtin:copy
    - name: standardize
      from: validated CSV
      to: final CSV
      desc: normalize amount and date fields
      resource: python:standardize.py
      depends:
        - load
```

设计约定：

- `design` 阶段产出 YAML/JSON 规格，不直接产出业务代码。
- CLI 本地 `[Unreleased]` 的 `design blueprint` 会给每个 state/step 默认写入 `resource: builtin:copy`，用于 Provider smoke test；实际业务实现生成后再替换为 `python:<script>` 等真实执行资源。
- `implement` 阶段读取 Blueprint/Specification，再生成目标语言实现。
- `execute`/Provider 阶段优先读取 `start_at/states`，旧代码仍可读取 `steps`。
- `resource` 是执行绑定字段；纯设计蓝图可以没有 `resource`，但 Provider run API 只执行带 `resource` 的步骤。
- 当前 Provider 先支持 `task` 状态；后续可扩展 `choice`、`parallel`、`map` 等状态类型。

## Provider 执行边界

Provider 本地 `[Unreleased]` 支持：

- `builtin:copy`：把当前输入复制到当前步骤输出，适合 smoke test 和演示。
- `python:<script>`：使用 `PYTHON_BIN` 或系统默认 Python 执行脚本。
- `bash:<script>`：使用 `bash` 执行脚本。

脚本资源必须位于 `PIPELINE_SCRIPT_DIR` 下；未设置时使用 Provider 进程当前工作目录。Provider 不执行任意 shell 字符串，避免把 Blueprint 变成开放式命令执行入口。路径校验会解析真实路径，拒绝 symlink/junction 指向允许目录之外。

路径边界：

- `PIPELINE_INPUT_DIR`：设置后，run API 的 `input_path` 必须位于该目录下。
- `PIPELINE_WORK_ROOT`：设置后，run API 的 `work_dir` 必须位于该目录下；未传 `work_dir` 时默认在该目录下按 job id 创建工作目录；该根目录不存在时 Provider 会自动创建。
- 两个变量未设置时保留本地开发体验，适合只在可信内网或本机演示使用。

```bash
curl -X POST http://localhost:8080/blueprints/sample/runs \
  -H "Content-Type: application/json" \
  -d '{"customer_id":"demo","input_path":"D:/tmp/raw.csv","work_dir":"D:/tmp/qtcloud-run"}'
```

执行结果会写入 Provider job store，可通过 `GET /process/jobs` 查看列表，通过 `GET /process/jobs/{id}` 查看单条详情。Pipeline 某一步失败时，job 会保留已完成步骤和失败步骤的输入、输出、资源与状态。Provider 服务启动时默认从 catalog 下的 `provider-jobs.json` 加载历史记录，路径优先级为 `JOB_STORE_PATH` -> `CATALOG_DIR/provider-jobs.json` -> `DATA_ROOT/catalog/provider-jobs.json` -> `.quanttide/data/catalog/provider-jobs.json`。后续版本再补更多状态类型、日志预览和真实云端传输闭环。
