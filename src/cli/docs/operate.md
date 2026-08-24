# operate — 安装与运行

本页记录本地安装和运行入口。

## 本地运行

```bash
cargo build --locked
qtcloud-data --help
qtcloud-data doctor --no-fail
qtcloud-data spec --help
qtcloud-data process --help
```

## 发布后安装

```bash
cargo install qtcloud-data-cli --version <X.Y.Z>
qtcloud-data --help
```

## 说明

以上入口用于确认二进制可启动、命令树可解析、基础 smoke 命令可执行。

