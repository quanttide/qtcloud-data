# 电商价格数据库 — qtcloud-data CLI 完整示例

用 `qtcloud-data` 生命周期命令完整复现 [docs/gallery/xmucpp](../../../docs/gallery/xmucpp/index.md) 案例（需求 → 规格 → 实现）。

## 运行方式

本示例使用 **mock LLM**（`scripts/mock_llm.py`，OpenAI 兼容 `/chat/completions`，按 prompt 关键词返回预设响应）——无需真实 API key，演示 LLM 可注入架构。有真实 key 时设 `LLM_API_KEY` 即可获得真实输出。

```bash
# 1. 启动 mock LLM
python3 scripts/mock_llm.py &

# 2. 环境变量（产物落在示例目录）
export LLM_BASE_URL=http://127.0.0.1:8899 LLM_API_KEY=mock
export DRD_DIR=.quanttide/data/drd SPEC_DIR=.quanttide/data/spec BLUEPRINT_DIR=.quanttide/data/blueprint
```

## 生命周期流程

```bash
# 需求：聊天记录 → DRD
qtcloud-data clarify from-chat chat.md

# 规格：DRD → Contract + Blueprint（LLM 输出 Markdown 表格，代码转 YAML）
qtcloud-data design contract .quanttide/data/drd/chat.md
qtcloud-data design blueprint .quanttide/data/drd/chat.md

# 规格（真实校验）：合并 contract 到 blueprint → wrap envelope → validate
# （design 分开发布 contract/blueprint，blueprint 内 contract 为空，需合并后 wrap）
python3 scripts/merge_contract.py
qtcloud-data spec validate .quanttide/data/spec/chat-blueprint.yaml
qtcloud-data spec wrap .quanttide/data/spec/chat-blueprint.yaml --output .quanttide/data/spec/chat-spec.yaml
qtcloud-data spec validate .quanttide/data/spec/chat-spec.yaml

# 实现：Blueprint → Python 脚本（LLM codegen）
qtcloud-data implement .quanttide/data/spec/chat-blueprint.yaml
```

## 产物

```
.quanttide/data/
├── drd/chat.md                    # 需求：DRD（clarify 产出）
└── spec/
    ├── chat-contract.yaml/.md     # 规格：Contract（design 产出）
    ├── chat-blueprint.yaml/.md/.html  # 规格：Blueprint + 可视化（design 产出）
    ├── chat-spec.yaml             # 规格：envelope（spec wrap 产出，validate 通过）
    └── chat-blueprint.py          # 实现：Python 脚本（implement 产出，3 步骤 + 组装）
```

## 与案例的对应

| 案例环节 | CLI 命令 | 产物 |
|---------|---------|------|
| 需求（两平台/每日/365天/关键词） | `clarify from-chat` | DRD |
| 规格-契约（缺失值 ≤30%） | `design contract` | Contract YAML |
| 规格-蓝图（3 采集器流水线） | `design blueprint` | Blueprint YAML + HTML |
| 规格校验（envelope） | `spec wrap` / `spec validate` | Specification YAML |
| 实现（每天每关键词 CSV） | `implement` | Python 脚本 |

> 后续环节（`process` 执行编排、`transfer` 打包交付）需真实采集脚本与存储平台凭证，见案例文档的边界说明。

## 备注（示例暴露的 CLI 真实行为）

- `design` 分开发布 contract/blueprint，**blueprint 内 contract 为空**——需合并后再 `wrap` 才能得到完整的 Specification（`scripts/merge_contract.py` 演示了合并）
- `spec validate` 是结构校验（schema/format/字段完整性），非数据质量校验（缺失率 ≤30% 属运行时检查）
- `implement` 未指定 `--output` 时产物写当前目录（`.py` 扩展名），示例已归置到 spec/
