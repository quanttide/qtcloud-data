# 开发指南

`qtcloud-provider` 是数据云的 Go 后端服务：读 CLI 生成的 Specification / Blueprint YAML，执行一次 pipeline，把这次 run 记成 job。它不产生数据、不碰飞书——只做「按定义跑一步处理，并留痕」。

怎么用、接口有哪些、环境变量怎么设，见 [README](../../README.md)——那几节现在由 README 承担，等接口拆细了再分出去。计划见[版本计划](roadmap.md)，工程待办见 [TODO](../../TODO.md)，已完成变更见 [CHANGELOG](../../CHANGELOG.md)。

## 代码落位

```text
src/provider/
├── cmd/qtcloud-provider/   服务入口：读环境变量、建 job store、装路由、监听 PORT
├── internal/
│   ├── api/                HTTP 路由与 handler：`GET /providers`、`GET /blueprints`、
│   │                       `GET /blueprints/{name}`、`POST /blueprints/{name}/runs`、
│   │                       `POST /transfer/send`、`POST /transfer/receive`、
│   │                       `GET /process/jobs`、`GET /process/jobs/{id}`、`GET /version`
│   ├── specstore/          读 Specification envelope 与 legacy Blueprint YAML
│   ├── pipeline/           执行器：状态机（start_at / states）与旧 steps 的线性兼容，
│   │                       builtin:copy 与受限脚本资源（python: / bash:）
│   ├── store/              job 记录与文件持久化（JOB_STORE_PATH 等四级定位）
│   └── provider/           传输实现（Dropbox / S3 stub），对应 StorageProvider
├── docs/dev-guide/         本文与版本计划
├── TODO.md                 工程待办
└── CHANGELOG.md            变更记录
```

## 一次 run 走哪几段

`POST /blueprints/{name}/runs` → `internal/api` 校验输入与工作目录边界（`PIPELINE_INPUT_DIR` / `PIPELINE_WORK_ROOT`）→ `specstore` 取 Blueprint 里的 pipeline → `internal/pipeline` 按 `start_at` 逐 state 执行，上一步输出文件作为下一步输入 → 每一步的结果（resource / input / output / status）写进 job → `store` 落盘 `provider-jobs.json`。

边界是硬约束，别松：脚本资源只能按参数化方式调解释器，脚本真实路径必须在 `PIPELINE_SCRIPT_DIR` 下（解析真实路径，拒 symlink 逃逸）；失败时 HTTP 只回泛化错误，细节进日志。

## 文档怎么分

按读者分三处：使用与接口在 README（暂未拆成 user-guide / api-references），开发与计划在本文和[版本计划](roadmap.md)。等用户侧真有人用起来，再把 README 里的「当前能力 / Pipeline 模型 / 执行资源 / 持久化与路径边界」拆进 `docs/api-references/`，运行步骤拆进 `docs/user-guide/`——先不建空目录占位。
