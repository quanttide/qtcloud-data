#!/usr/bin/env python3
"""把 design contract 产出的 contract 字段合入 blueprint（design 分开发布，wrap 前需合并）。"""
import yaml, sys

base = sys.argv[1] if len(sys.argv) > 1 else ".quanttide/data/spec"
with open(f"{base}/chat-blueprint.yaml") as f:
    bp = yaml.safe_load(f)
with open(f"{base}/chat-contract.yaml") as f:
    contract = yaml.safe_load(f)
bp["contract"] = contract["contract"]
with open(f"{base}/chat-blueprint.yaml", "w") as f:
    yaml.safe_dump(bp, f, allow_unicode=True, sort_keys=False)
print("✓ contract 已合入 blueprint")
