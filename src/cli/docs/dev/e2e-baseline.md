# 基线 e2e — 验证记录

> 目的：为现有 `process` 命令建立全链路回归保护，先于 v0.2.1 内部重构落地。

## 覆盖范围

| 维度 | 内容 |
|------|------|
| 链路 | `process` 全链路：receive → pipeline → send |
| 数据 | 真实业务 fixture（`tests/fixtures/github-activity/`，对标 GitHub 用户活动面板） |
| 内容断言 | 产物 `final.csv` 与 `expected-final.csv` 逐字节一致（脱敏 + 排序 + 标准表头） |
| 落盘记录 | `jobs.json`（delivered / source_url 脱敏）、`registry.json`（process 产物登记）、日志含 `pipeline completed` |
| 安全 | URL query token 不出现在 stdout / stderr / jobs.json |

## fixture 形态

```text
tests/fixtures/github-activity/
├── raw.csv            未脱敏活动明细（含 login 冗余列、未排序）
├── normalize.sh       真实转换脚本：用户 ID → 去标识化序号，排序，标准表头
└── expected-final.csv 期望产物
```

流水线脚本做实际业务转换（脱敏 + 排序），而非 `copy`——保证内容级断言有效。
Windows 本地退化为 copy 脚本（仅回归链路机制），CI（ubuntu-latest）使用完整脚本。

## 运行方式

```bash
cargo build
cargo test --test e2e_baseline
```

## 验证记录

| 日期 | 结果 | 说明 |
|------|------|------|
| 2026-08-02 | ✅ 通过 | `e2e_process_full_chain_delivers_normalized_activity` 1 passed；全量 `cargo test` 115+19+1+10+8 = 153 全绿（v0.2.1 发布前） |

## 后续扩展（业务 e2e，待 Provider 稳定后排期）

基线 e2e 只验证现有 `process` 全链路机制。业务链路（raw + map.dta → review_master）
依赖 Provider 的 merge_review / export（Provider ROADMAP [0.0.3]），落地后补测。

## 覆盖率基线（v0.2.2 补测后）

| 版本 | 整体行覆盖 | 说明 |
|------|-----------|------|
| v0.2.1 | 53.2% | 基线（`cargo llvm-cov test --workspace`，不含子进程 CLI 覆盖） |
| v0.2.2 第一轮 | 62.5% | error 100% / version 96% / transfer 80% / google_drive 80% / onedrive 77% / contract 67% / s3 43% |
| v0.2.2 第二轮（Handler 注入） | 71.4% | clarify 80% / implement 82% / review 66% / design 63% / blueprint_core 95% |
| v0.2.1 错误收尾后 | 73.8% | 全部命令统一 CliError，错误路径可测 |
| v0.2.1 补测（fake cue + doctor） | **83.7%** | blueprint 85% / pipeline 82% / doctor 89% / catalog 93% / spec 93% / transfer 84% |

补测方式：`cargo llvm-cov test --workspace`（lib + harness 数据）。
子进程 CLI 覆盖（main.rs 等）需要插桩二进制 + 手动合并 profraw。

LLM 命令测试：命令模块改为 `XxxHandler { llm }` 构造器注入，测试复用
quanttide-agent 的 `HttpClient` 抽象构造 fake（`lib.rs test_support::fake_llm`），
不发起真实网络请求，生产路径 `LLM::default()` 行为不变。

剩余 0% 模块：main（CLI 分发，仅子进程插桩流可测）、baidu_drive/sftp（需要真实服务，需 CI 起 sshd 或本地模拟）。
