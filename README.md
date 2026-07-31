# 量潮数据云

量潮数据云是面向内部 DataOps 工作流的多端工程。当前仓库采用 monorepo 结构，把 CLI、Provider 和 Studio 放在同一仓库中，通过 Git 里的约定文件维护规划、执行和发布状态。

## 项目结构

```text
qtcloud-data/
├── contract.yaml              # scope 契约事实源
├── README.md                  # 仓库总览
├── RELEASE_STATUS.md          # 发布状态事实源
├── .github/workflows/         # CI / Release 自动化
└── src/
    ├── cli/                   # Rust 命令行工具
    ├── provider/              # Go 后端服务
    ├── studio/                # Flutter 前端应用
    └── test/                  # 测试 fixture / 示例数据
```

## 当前范围

| Scope | 语言 | 构建工具 | 当前状态 |
|---|---|---|---|
| `cli` | Rust | Cargo | v0.2.0 源码已合并，crates.io 发布待 owner 权限 |
| `provider` | Go | Go toolchain | v0.2.0 源码已合并，提供 Blueprint runner 底座 |
| `studio` | Dart / Flutter | Flutter | 原型阶段，本次 CLI + Provider 交付不包含 Studio 发布 |

各 scope 的规划、执行拆解和变更记录分别维护在对应目录：

- `src/cli/ROADMAP.md`、`src/cli/TODO.md`、`src/cli/CHANGELOG.md`
- `src/provider/ROADMAP.md`、`src/provider/TODO.md`、`src/provider/CHANGELOG.md`
- `src/studio/ROADMAP.md`

## CLI

```powershell
cd src/cli
cargo build
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

源码安装：

```powershell
cd src/cli
cargo install --path .
qtcloud-data --help
```

## Provider

```powershell
cd src/provider
go run ./cmd/qtcloud-provider
go test ./...
go vet ./...
```

Provider 读取 CLI 生成的 Specification / Blueprint YAML，并提供 `/blueprints`、`/blueprints/{name}`、`/blueprints/{name}/runs`、`/process/jobs` 等接口。

## Studio

```powershell
cd src/studio
flutter pub get
flutter run
flutter test
```

## 发布约定

- 多 scope tag 使用前缀，例如 `cli/v0.2.0`、`provider/v0.2.0`、`studio/v0.1.0-alpha.1`。
- 发布前需要确认配置文件版本、CHANGELOG 条目、Git tag 和 GitHub Release 一致。
- CLI 发布到 crates.io 后，用户可通过 `cargo install qtcloud-data-cli` 安装最新正式版本。

## License

Apache 2.0
