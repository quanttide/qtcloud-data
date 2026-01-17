# QuantTide 数据清洗工程规范

## 概述

本规范定义了 QuantTide 数据清洗项目的标准化结构和交付物要求，基于 `quanttide_data_toolkit` 实现，为数据清洗项目提供统一的工程框架。

## 适用范围

- **领域**：结构化问卷数据清洗
- **平台**：qtcloud_data、qtcloud_knowl 等共享领域模型
- **角色**：项目经理、数据架构师、数据工程师、质量保证

## 核心原则

1. **理解先于处理**：在编写清洗代码前，先重建数据的"出生证明"
2. **规范先于代码**：所有交付物必须符合标准结构
3. **可追溯性**：保留所有中间步骤，可追溯修改原因
4. **业务共识**：Codebook 是业务与技术团队的共识协议

## 工作区结构

### 目录布局

```
workspace/
├── plan/              # 业务意图文件
├── spec/              # 工程规范文件
├── schema/            # 数据结构定义
├── processor/         # 数据处理器
├── inspector/        # 数据检查器
├── record/            # 数据记录
├── report/            # 质量报告
└── manifest/          # 交付物清单
```

### 组件说明

| 组件 | 说明 | 必需文件 | 格式 |
|-----|------|---------|------|
| plan | 业务意图文件 | `{项目}_plan.md` | Markdown |
| spec | 工程规范文件 | `{项目}_spec.md`, `base_spec.md` | Markdown |
| schema | 数据结构定义 | `{项目}_schema.json` | JSON |
| processor | 数据处理器 | `{项目}_cleaner.py` | Python |
| inspector | 数据检查器 | `{项目}_inspector.py` | Python |
| record | 数据记录 | `{项目}_raw.csv`, `{项目}_cleaned.csv` | CSV |
| report | 质量报告 | `{项目}_report.md` | Markdown |
| manifest | 交付物清单 | `{项目}_manifest.json` | JSON |

## 领域模型规范

### Workspace 工作区

工作区是数据清洗项目的完整环境，包含所有必需的组件。

**验证方法**：
- `is_complete()`: 检查工作区是否完整
- `validation_report()`: 生成验证报告
- `get_component()`: 获取组件路径

**必需组件**：
- plan, spec, schema, processor, inspector, record, report, manifest

### BusinessArtifacts 业务工件

业务工件是数据清洗项目中的关键交付物。

**验证方法**：
- `plan_is_clear()`: Plan 清晰度检查
- `schema_is_complete()`: Schema 完整性检查
- `schema_is_well_formed()`: Schema 格式检查
- `manifest_is_complete()`: Manifest 完整性检查

### DataSchema 数据结构

数据结构定义，描述清洗后数据的完整规格。

**验证方法**：
- `is_complete()`: Schema 完整性检查
- `is_well_defined()`: Schema 定义检查
- `fields_match()`: 字段匹配检查

**必需字段**：
- name, version, schema, quality_rules, transformations
- schema.fields: 字段定义列表

**字段属性**：
- name: 字段名称
- type: 字段类型（string/integer/float/binary/datetime/categorical/text）
- required: 是否必填
- missing_codes: 缺失值编码

### DataPipeline 数据流水线

数据流水线是数据清洗和验证的完整流程。

**验证方法**：
- `inspector_available()`: Inspector 可用性检查
- `inspector_complies_with_schema()`: Schema 合规性检查
- `data_quality_acceptable()`: 数据质量可接受性检查
- `business_rules_complied()`: 业务规则合规性检查

## 交付物规范

### 1. Plan 业务意图文件

**文件格式**：`{项目}_plan.md`

**必需章节**：
```markdown
## 数据模型
[字段定义、变量类型、值标签]

## 数据处理流程
[清洗步骤、转换规则、逻辑约束]
```

**内容要求**：
- 描述清洗目标和业务背景
- 定义所有字段的标准名称和类型
- 说明数据转换规则和逻辑约束

### 2. Spec 工程规范文件

**文件格式**：`{项目}_spec.md`

**必需内容**：
- 适用范围说明
- 核心目标定义
- 标准化清洗流程
- 角色分工建议

**示例**：参考 `questionnaire_cleaning_spec.md`

### 3. Schema 数据结构定义

**文件格式**：`{项目}_schema.json`

**JSON 结构**：
```json
{
  "name": "数据集名称",
  "version": "版本号",
  "schema": {
    "fields": [
      {
        "name": "字段名",
        "type": "字段类型",
        "description": "字段描述",
        "required": true,
        "missing_codes": [-99, -88]
      }
    ]
  },
  "quality_rules": [
    {
      "field": "字段名",
      "rule": "规则描述",
      "valid_values": [...]
    }
  ],
  "transformations": [
    {
      "from": "原始字段",
      "to": "清洗后字段",
      "type": "转换类型"
    }
  ]
}
```

### 4. Processor 数据处理器

**文件格式**：`{项目}_cleaner.py`

**必需方法**：
- `clean(raw_data: pd.DataFrame) -> pd.DataFrame`: 执行数据清洗
- `transform(raw_data: pd.DataFrame) -> pd.DataFrame`: 执行数据转换

**代码要求**：
- 使用 `quanttide_data_toolkit` 中的领域模型
- 包含详细的注释说明清洗逻辑
- 支持可复现的清洗流程

### 5. Inspector 数据检查器

**文件格式**：`{项目}_inspector.py`

**必需方法**：
- `validate_schema_compliance(data: pd.DataFrame) -> dict`: Schema 合规性检查
- `validate_data_quality(data: pd.DataFrame) -> dict`: 数据质量检查
- `validate_business_rules(data: pd.DataFrame) -> dict`: 业务规则检查

**返回格式**：
```python
{
  "status": "PASS" | "FAIL",
  "issues": [],
  "checks": [...]
}
```

### 6. Record 数据记录

**文件格式**：
- `{项目}_raw.csv`: 原始数据
- `{项目}_cleaned.csv`: 清洗后数据

**CSV 要求**：
- UTF-8 编码
- 时间字段统一为 `YYYY-MM-DD HH:MM:SS` 格式
- 缺失值使用预定义编码（如 -99、-88）

### 7. Report 质量报告

**文件格式**：`{项目}_report.md`

**必需章节**：
```markdown
## 概述
[项目背景、清洗目标]

## 1. 数据概览
[原始数据规模、样本量、字段数]

## 2. 数据转换说明
[字段映射、转换规则]

## 3. 数据统计
[字段分布、统计摘要]

## 4. 数据质量检查
[质量检查结果、通过/失败]

## 5. 异常记录说明
[异常情况说明、处理方式]

## 6. 数据交付物清单
[交付文件列表]
```

### 8. Manifest 交付物清单

**文件格式**：`{项目}_manifest.json`

**JSON 结构**：
```json
{
  "order_id": "订单ID",
  "customer": "客户名称",
  "project_name": "项目名称",
  "created_at": "创建时间",
  "updated_at": "更新时间",
  "status": "completed",
  "includes": [
    {
      "type": "recipe|dataset|plan|schema|inspector|report",
      "file": "文件路径",
      "id": "文件ID",
      "description": "文件描述"
    }
  ],
  "quality_assurance": {
    "status": "passed",
    "inspector_result": {
      "schema_compliance": "PASS",
      "data_quality": "PASS",
      "business_rules": "PASS"
    }
  }
}
```

## 标准化清洗流程

### 阶段 1：需求对齐与数据探查

**输入**：原始数据、业务需求

**动作**：
- 与客户确认清洗目标、输出格式、关键业务规则
- 数据探查，识别数据质量问题

**产出**：签署版 Codebook + 数据问题清单

### 阶段 2：元数据标准化

**目标**：统一基础数据格式，消除技术噪声

**关键操作**：
- 时间字段 → 统一为 `YYYY-MM-DD HH:MM:SS`
- 文本编码 → 统一为 UTF-8
- 数值字段 → 剥离单位、符号

### 阶段 3：结构化清洗

**1. 分类变量处理**
- 将文本选项映射为预定义数值编码

**2. 多选题拆分**
- 将单列多选拆分为多个虚拟变量

**3. 逻辑一致性校验**
- 检查跳转逻辑是否被违反

**4. 缺失值精细化处理**
- 区分系统缺失（-88）和用户跳过（-99）
- 禁止直接删除或均值填充，除非客户明确要求

### 阶段 4：质量验证

使用 `quanttide_data_toolkit.DataPipeline` 执行：
- 完整性检查：字段数、样本量是否符合预期
- 分布合理性：数值范围是否合理
- 逻辑验证：业务规则是否被遵守

### 阶段 5：交付

**交付清单**：
- 清洗后数据（CSV/Excel）
- 可复现清洗代码（含详细注释）
- 最终版 Codebook
- 异常处理日志（记录所有人工干预点）

## 角色分工

| 角色 | 责任 |
|-----|------|
| 项目经理 | 主导需求沟通，协调客户确认 Codebook，定义交付范围 |
| 数据架构师 | 提供问卷逻辑说明、跳转规则、业务背景 |
| 数据工程师 | 基于 Codebook 编写清洗代码，执行质量验证 |
| 质量保证 | 抽样核对清洗结果与 Codebook 一致性 |

## 命名规范

### 目录命名
- 数据文件夹：`data`（被 `.gitignore` 过滤）
- 交付文件夹：`dist`（被 `.gitignore` 过滤）
- 数据库、文件夹、仓库：统一使用**单数**命名

### 文件命名
- 计划文件：`{项目}_plan.md`
- 规格文件：`{项目}_spec.md`
- 结构定义：`{项目}_schema.json`
- 清洗代码：`{项目}_cleaner.py`
- 检查代码：`{项目}_inspector.py`
- 数据文件：`{项目}_raw.csv`, `{项目}_cleaned.csv`
- 质量报告：`{项目}_report.md`
- 交付清单：`{项目}_manifest.json`

## 验证检查清单

### 工作区验证
- [ ] 所有 8 个必需组件目录都存在
- [ ] 每个组件目录包含至少一个文件

### 业务工件验证
- [ ] Plan 包含"数据模型"和"数据处理流程"章节
- [ ] Schema 包含所有必需字段
- [ ] Schema 字段定义完整
- [ ] Manifest 包含所有必需字段

### 数据结构验证
- [ ] Schema 完整性检查通过
- [ ] Schema 定义检查通过
- [ ] 字段匹配检查通过

### 数据流水线验证
- [ ] Inspector 可用
- [ ] Schema 合规性检查通过
- [ ] 数据质量检查通过
- [ ] 业务规则检查通过

## 常见陷阱提醒

- ❌ 假设"空值 = 未回答"（可能实际是"不适用"）
- ❌ 忽略反向计分题，导致量表方向错误
- ❌ 用"0"填充缺失，扭曲统计结果
- ❌ 未保留中间步骤，无法追溯修改原因
- ❌ 跳过 Codebook 确认，直接开始编码

## 版本历史

| 版本 | 日期 | 说明 |
|-----|------|------|
| 1.0.0 | 2025-01-17 | 初始版本，基于 quanttide_data_toolkit |

## 参考

- [QuantTide 数据清洗工具包](../quanttide_data_toolkit/)
- [问卷清洗示例](../../tests/fixtures/workspace/)
