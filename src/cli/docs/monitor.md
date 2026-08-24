# monitor — 发布后验证

本页记录发布后要看的验证项，以及当前已完成的本地检查。

## 已完成

- `cargo fmt --check`
- `cargo test --locked`
- `cargo clippy --locked -- -D warnings`
- `qtcloud-devops release audit -v cli/v0.3.0 --scope cli`

## 当前结果

- 格式检查：通过
- 测试：通过
- clippy：通过
- release audit：部分通过，仍有工作区、标签和 GitHub Release 不一致问题

## 发布后还要补

- `qtcloud-devops release status`
- `gh release view cli/v0.3.0`
- `cargo info qtcloud-data-cli --registry crates-io`
- `cargo install qtcloud-data-cli --version <X.Y.Z>`

