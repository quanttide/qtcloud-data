# 数据格式（catalog.rs + store.rs）

本文档对应 `src/catalog.rs` 与 `src/store.rs`。定义 `.quanttide/data/catalog/` 下
三个 JSON 落盘文件的字段级格式——Studio、Provider 与 CLI 共同消费这些文件，**格式变更需保持兼容**。

## store 模块（共享基础设施）

`store.rs` 提供三处统一（v0.2.1 收敛）：

- `catalog_dir()`：路径解析优先级 `CATALOG_DIR` > `DATA_ROOT/catalog` > `.quanttide/data/catalog`
- `now_utc()`：UTC 时间格式化（RFC 3339）
- `Registry<T>`：JSON registry 读写（open/get/entries/len/is_empty/insert/remove/save），
  **原子写盘**（临时文件 + rename，避免半写损坏）

所有落盘文件均为 pretty JSON，顶层为 key → record 的对象映射。

## registry.json（catalog 产物登记）

`catalog add` / `process` 成功交付时写入。

```json
{
  "ABC-001": {
    "name": "ABC-001",
    "path": "/abs/path/to/final.csv",
    "size": 12345,
    "received_at": "2026-08-02T00:00:00Z",
    "provider": "process",
    "source": "process:job-id",
    "status": "delivered"
  }
}
```

### Volume 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `name` | string | 登记名（默认取文件名） |
| `path` | string | 规范化绝对路径 |
| `size` | integer | 字节数 |
| `received_at` | string | UTC 接收时间 |
| `provider` | string? | 来源 provider（缺省省略） |
| `source` | string? | 来源描述（缺省省略） |
| `status` | enum | 见下 |

### VolumeStatus 枚举

序列化 `snake_case`，保持 v0.2.0 以来的落盘字符串：

| 值 | 含义 |
|----|------|
| `received` | 已接收原始数据（默认） |
| `processing` | 处理中 |
| `processed` | 已处理 |
| `delivered` | 已交付 |
| `unknown` | 未知状态（`#[serde(other)]` 兼容旧数据/未来新增状态） |

**兼容约定**：`#[serde(other)]` 保证未知状态字符串不会导致整表反序列化失败；
`#[serde(default)]` 保证旧记录缺 `status` 字段时回退 `received`。

## jobs.json（process job 记录）

`process` 成功或失败后写入，日志对应 `CATALOG_DIR/jobs/<job-id>.log`。

```json
{
  "job-id": {
    "id": "job-id",
    "customer_id": "ABC-001",
    "source_url": "https://share.example/file.csv",   // 已脱敏（无 query token）
    "blueprint": "github-activity",
    "pipeline": "normalize",
    "raw_path": "/workdir/raw.csv",
    "output_path": "/workdir/final.csv",
    "link_path": "/catalog/share-link.txt",
    "log_path": "/catalog/jobs/job-id.log",
    "status": "delivered",          // delivered | failed
    "started_at": "2026-08-02T00:00:00Z",
    "finished_at": "2026-08-02T00:01:00Z"
  }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | job id |
| `customer_id` | string | 客户标识 |
| `source_url` | string | 脱敏后的来源 URL（移除 query 与 fragment，不含 token） |
| `blueprint` | string? | 使用的 blueprint（缺省省略） |
| `pipeline` | string | pipeline 名称 |
| `raw_path` / `output_path` | string | 原始/最终产物路径 |
| `link_path` | string | 分享链接文件路径 |
| `log_path` | string | 执行日志路径 |
| `status` | string | `delivered` / `failed` |
| `started_at` / `finished_at` | string | UTC 起止时间 |

## delivery-links.json（transfer 交付链接）

`transfer send` 成功后写入（使用 `--output` 时链接同时写入指定文件）。

```json
{
  "report-1710000000000": {
    "id": "report-1710000000000",
    "provider": "dropbox",
    "file_path": "/abs/path/report.csv",
    "remote_path": "/send/report.csv",
    "link": "https://www.dropbox.com/s/xxx/report.csv?dl=1",
    "link_path": "/catalog/share-link.txt",
    "status": "sent",
    "sent_at": "2026-08-02T00:00:00Z"
  }
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | string | 文件名 stem（sanitize）+ 毫秒时间戳 |
| `provider` | string | 传输平台 |
| `file_path` | string | 源文件规范化路径 |
| `remote_path` | string | 远程路径 |
| `link` | string | 分享链接 |
| `link_path` | string? | `--output` 链接文件路径（未指定时省略） |
| `status` | string | `sent` |
| `sent_at` | string | UTC 发送时间 |

## 落盘格式兼容约定

1. **字段名与枚举字符串稳定**：`registry.json` / `jobs.json` / `delivery-links.json` 的字段名、
   `VolumeStatus` 的枚举字符串不得变更（Studio / Provider 在读取）
2. **向后兼容优先**：新增字段用 `Option` + `skip_serializing_if`；未知枚举值用 `#[serde(other)]`
3. **原子写盘**：所有 registry 写入走临时文件 + rename，杜绝半写
4. **路径规范化**：`file_path` / `path` 记录规范化绝对路径（`canonicalize`，失败回退原始路径）
