# CLI Release Process

This process follows the desktop `quanttide-tutorial-of-devops` conventions:

```text
plan -> code -> build -> test -> release -> deploy -> operate -> monitor
```

For the CLI, the current repository automates the `plan` through `release` stages. The later stages are recorded as follow-up work in `ROADMAP.md`.

## Release Principles

- Use small, reviewable releases.
- Use SemVer. For this multi-component repository, CLI tags use `cli/vX.Y.Z`.
- Treat tags as immutable. Never move an existing release tag.
- Treat `CHANGELOG.md` as the release fact source.
- Generate GitHub Release notes from the matching CHANGELOG entry.
- Do not publish crates manually from a developer laptop. Use `qtcloud-devops release publish` as the release entrypoint.

## Operation Checks

Before release work starts:

1. `src/cli/Cargo.toml` contains the target version.
2. `src/cli/CHANGELOG.md` contains `## [X.Y.Z] - YYYY-MM-DD`.
3. The target tag does not already exist.
4. The working tree is clean after the release-prep commit.
5. The change has gone through the feature branch, Pull Request, review, and `main` merge flow.
6. Build, test, clippy, and format checks pass.

## Operation Flow

### 1. Plan

Update the version milestone in `src/cli/ROADMAP.md` and break remaining work into `src/cli/TODO.md`.

```bash
qtcloud-devops plan status --scope cli
qtcloud-devops plan audit --scope cli
```

### 2. Code

Make the implementation and documentation changes on a feature branch. Keep the release scope small and reviewable.

```bash
git switch -c codex/cli-v0.2.0-release
qtcloud-devops code audit src/cli
```

### 3. Build and Test

```bash
qtcloud-devops build status
qtcloud-devops test status

cd src/cli
cargo fmt --check
cargo build --locked
cargo test --locked
cargo clippy --locked -- -A warnings
```

### 4. Update Release Records

Update the package version in `src/cli/Cargo.toml` and add the release entry to `src/cli/CHANGELOG.md`:

```markdown
## [0.2.0] - 2026-08-01

### Added
- Release change description.

### Fixed
- Release fix description.
```

Update the matching checkbox items in `src/cli/ROADMAP.md` and `src/cli/TODO.md`.

### 5. Commit and Review

```bash
git diff --name-only
git add \
  .github/workflows/release-cli.yml \
  .github/workflows/test-cli.yml \
  src/cli/Cargo.toml \
  src/cli/CHANGELOG.md \
  src/cli/README.md \
  src/cli/ROADMAP.md \
  src/cli/TODO.md \
  src/cli/docs/dev/release.md
git status --short
git commit -m "chore(cli): prepare v0.2.0 release"
git push -u origin codex/cli-v0.2.0-release
```

The staged file list should stay inside the CLI release scope. If Provider, Studio, or shared DevOps docs also need planning cleanup, prepare a separate branch and Pull Request for that scope. Run `git status --short` before committing; only unrelated local scratch directories should remain unstaged.

Create a Pull Request, complete code review, and merge the feature branch into `main`. Release tags must point to commits reachable from `main`.

### 6. Release Preflight

After the merge, use the DevOps CLI from a clean `main` checkout:

```bash
qtcloud-devops release status
qtcloud-devops release audit -v cli/v0.2.0 --scope cli
qtcloud-devops release publish -v cli/v0.2.0 --registry crates --dry-run
```

The dry run must not create a tag, GitHub Release, or crates.io version.

### 7. Publish

Only after maintainer confirmation, run the DevOps release entrypoint:

```bash
qtcloud-devops release publish -v cli/v0.2.0 --registry crates -y
```

The command creates and pushes `cli/v0.2.0`. The `release-cli.yml` GitHub Actions workflow then performs the registry publish step:

1. Checks the tag, Cargo version, CHANGELOG entry, clean checkout, and `main` ancestry.
2. Builds and tests the package.
3. Publishes `qtcloud-data-cli` to crates.io using `CRATES_API_TOKEN`.
4. Builds Linux and Windows binaries.
5. Creates or updates the GitHub Release with notes extracted from CHANGELOG.

### 8. Verify

```bash
cargo info qtcloud-data-cli --registry crates-io
cargo install qtcloud-data-cli --version 0.2.0
qtcloud-data doctor --no-fail
qtcloud-data spec --help
qtcloud-data process --help
```

## Local Cargo Mirror

If Cargo is configured to replace crates.io with `rsproxy`, target crates.io explicitly during local dry runs. This command is only a package preflight; it must not be used for the real release:

```bash
cd src/cli
cargo publish --locked --dry-run --registry crates-io --allow-dirty
```
