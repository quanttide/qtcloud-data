# tests/fixtures — e2e 测试夹具

本目录存放真实项目形态的 smoke/e2e 夹具（v0.2.1 基线回归 + v0.2.2 业务链路）。

## 结构

| 路径 | 用途 |
|------|------|
| `github-activity/raw.csv` | 原始输入：未脱敏的活动明细（对标 `examples/AI范例-DRD-数据需求文档.md` 的 GitHub 用户活动面板） |
| `github-activity/normalize.sh` | 流水线脚本：脱敏（用户 ID → 去标识化序号）+ 排序 + 标准表头 |
| `github-activity/expected-final.csv` | 期望产物：脱敏、排序后的最终交付 CSV |

## 约定

- 夹具数据保持“真实项目”形态：含脱敏前字段、未排序、含冗余列，流水线脚本做实际业务转换（而非 copy）。
- 流水线脚本默认 unix shell（CI 为 ubuntu-latest）；Windows 仅本地开发，测试按平台落盘脚本。
- `expected-final.csv` 作为内容级断言基准：产物必须与之一致（逐字节）。

## 基线 e2e 断言（`tests/e2e_baseline.rs`）

- 链路：`process` 全链路（receive → pipeline → send），fixture 驱动假 qtdata 读 `$FIXTURE_RAW`。
- 产物：`final.csv` 与 `expected-final.csv` 逐字节一致。
- 落盘记录：`jobs.json` 单条 `delivered` 且 `source_url` 脱敏（不含 `access_token=secret`）；`registry.json` 登记 provider=`process`、status=`delivered`；日志含 `pipeline completed`。
- 安全：URL query token 不出现在 stdout / stderr / jobs.json。
