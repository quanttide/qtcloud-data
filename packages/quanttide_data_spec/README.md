# QuantTide 数据清洗工程规范

## 简介

`quanttide_data_spec` 定义了 QuantTide 数据清洗项目的标准化结构和交付物要求。它是基于 `quanttide_data_toolkit` 实现的工程标准，为数据清洗项目提供统一的框架。

## 适用场景

- **问卷数据清洗**：将原始问卷数据转化为标准化、可分析的数据集
- **结构化数据清洗**：基于规则的结构化数据处理
- **数据质量保证**：验证数据清洗结果的正确性和完整性

## 核心概念

### 工作区 (Workspace)

数据清洗项目的完整环境，包含所有必需的组件：

```
workspace/
├── plan/          # 业务意图文件
├── spec/          # 工程规范文件
├── schema/        # 数据结构定义
├── processor/     # 数据处理器
├── inspector/   # 数据检查器
├── record/        # 数据记录
├── report/        # 质量报告
└── manifest/      # 交付物清单
```

### 领域模型

基于 `quanttide_data_toolkit` 提供的领域模型：

- **Workspace**: 工作区完整性验证
- **BusinessArtifacts**: 业务工件验证
- **DataSchema**: 数据结构定义
- **DataPipeline**: 数据流水线验证

## 快速开始

### 1. 安装依赖

```bash
pip install quanttide-data-toolkit
```

### 2. 创建工作区

```python
from pathlib import Path
from quanttide_data_toolkit import Workspace

# 创建工作区实例
workspace = Workspace(Path("/path/to/workspace"))

# 验证工作区完整性
if workspace.is_complete():
    print("工作区完整")
else:
    print(workspace.validation_report())
```

### 3. 验证业务工件

```python
from quanttide_data_toolkit import BusinessArtifacts

# 创建业务工件验证器
artifacts = BusinessArtifacts(workspace)

# 验证各业务工件
print(f"Plan 清晰: {artifacts.plan_is_clear()}")
print(f"Schema 完整: {artifacts.schema_is_complete()}")
print(f"Manifest 完整: {artifacts.manifest_is_complete()}")
```

### 4. 验证数据流水线

```python
from quanttide_data_toolkit import DataPipeline

# 创建数据流水线验证器
pipeline = DataPipeline(workspace)

# 验证数据流水线
print(f"Inspector 可用: {pipeline.inspector_available()}")
print(f"Schema 合规: {pipeline.inspector_complies_with_schema()}")
print(f"质量可接受: {pipeline.data_quality_acceptable()}")
print(f"规则合规: {pipeline.business_rules_complied()}")
```

## 目录结构

```
packages/
├── quanttide_data_toolkit/    # 领域模型库
│   ├── src/
│   │   └── quanttide_data_toolkit/
│   │       ├── workspace.py
│   │       ├── artifacts.py
│   │       ├── data_schema.py
│   │       └── data_pipeline.py
│   └── tests/
│       └── quanttide_data_toolkit/
│           ├── test_workspace.py
│           ├── test_artifacts.py
│           ├── test_data_schema.py
│           └── test_data_pipeline.py
│
└── quanttide_data_spec/        # 工程规范（当前）
    ├── data_cleaning_spec.md   # 数据清洗规范
    └── README.md               # 本文件
```

## 规范文档

完整的技术规范请参考 [data_cleaning_spec.md](data_cleaning_spec.md)，包含：

- 工作区结构定义
- 交付物规范
- 领域模型使用指南
- 标准化清洗流程
- 验证检查清单

## 交付物清单

每个数据清洗项目应包含以下交付物：

| 组件 | 文件 | 说明 |
|-----|------|------|
| Plan | `{项目}_plan.md` | 业务意图文件 |
| Spec | `{项目}_spec.md` | 工程规范文件 |
| Schema | `{项目}_schema.json` | 数据结构定义 |
| Processor | `{项目}_cleaner.py` | 数据处理器 |
| Inspector | `{项目}_inspector.py` | 数据检查器 |
| Record | `{项目}_raw.csv`, `{项目}_cleaned.csv` | 数据记录 |
| Report | `{项目}_report.md` | 质量报告 |
| Manifest | `{项目}_manifest.json` | 交付物清单 |

## 示例项目

参考 `tests/fixtures/workspace/` 中的示例项目：

```
tests/fixtures/workspace/
├── plan/
│   └── questionnaire_cleaning_plan.md
├── spec/
│   ├── base_spec.md
│   └── questionnaire_cleanning_spec.md
├── schema/
│   └── questionnaire_schema.json
├── processor/
│   └── questionnaire_cleaner.py
├── inspector/
│   └── questionnaire_inspector.py
├── record/
│   ├── questionnaire_raw.csv
│   └── questionnaire_cleaned.csv
├── report/
│   └── questionnaire_cleaning_report.md
└── manifest/
    ├── questionnaire_cleaning_manifest.json
    ├── questionnaire_cleaning_recipe_manifest.json
    └── questionnaire_dataset_manifest.json
```

## 验证检查

使用 `quanttide_data_toolkit` 自动验证工作区：

```python
from quanttide_data_toolkit import Workspace, BusinessArtifacts, DataPipeline
from pathlib import Path

workspace = Workspace(Path("workspace"))

# 1. 验证工作区完整性
assert workspace.is_complete(), "工作区不完整"

# 2. 验证业务工件
artifacts = BusinessArtifacts(workspace)
assert artifacts.plan_is_clear(), "Plan 不清晰"
assert artifacts.schema_is_complete(), "Schema 不完整"
assert artifacts.manifest_is_complete(), "Manifest 不完整"

# 3. 验证数据流水线
pipeline = DataPipeline(workspace)
assert pipeline.inspector_available(), "Inspector 不可用"
assert pipeline.inspector_complies_with_schema(), "Schema 不合规"
assert pipeline.data_quality_acceptable(), "数据质量不可接受"
assert pipeline.business_rules_complied(), "业务规则不合规"

print("✅ 所有验证通过")
```

## 核心原则

1. **理解先于处理**：在编写清洗代码前，先重建数据的"出生证明"
2. **规范先于代码**：所有交付物必须符合标准结构
3. **可追溯性**：保留所有中间步骤，可追溯修改原因
4. **业务共识**：Codebook 是业务与技术团队的共识协议

## 角色分工

| 角色 | 责任 |
|-----|------|
| 项目经理 | 主导需求沟通，协调客户确认 Codebook，定义交付范围 |
| 数据架构师 | 提供问卷逻辑说明、跳转规则、业务背景 |
| 数据工程师 | 基于 Codebook 编写清洗代码，执行质量验证 |
| 质量保证 | 抽样核对清洗结果与 Codebook 一致性 |

## 常见陷阱

- ❌ 假设"空值 = 未回答"（可能实际是"不适用"）
- ❌ 忽略反向计分题，导致量表方向错误
- ❌ 用"0"填充缺失，扭曲统计结果
- ❌ 未保留中间步骤，无法追溯修改原因
- ❌ 跳过 Codebook 确认，直接开始编码

## 参考资源

- [QuantTide 数据清洗工具包](../quanttide_data_toolkit/)
- [数据清洗规范文档](data_cleaning_spec.md)
- [示例项目](../../tests/fixtures/workspace/)

## 版本历史

| 版本 | 日期 | 说明 |
|-----|------|------|
| 1.0.0 | 2025-01-17 | 初始版本 |

## 贡献

如有问题或建议，请参考项目贡献指南。
