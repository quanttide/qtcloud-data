# 数据清洗工作手册

## 概述

本手册基于问卷数据清洗的QA测试流程，定义数据清洗项目的标准工作流和交付物结构。

## 核心概念

### 角色

- **委托人**: 提供原始数据的一方
- **代理人**: 提供清洗规范的一方
- **系统**: 自动化执行清洗流程的工具平台

### 目录结构

```
project_name/
├── spec/           # 规范（系统预设）
│   ├── base.md
│   └── questionnaire_cleaning.md
├── record/         # 原始数据（委托人提供）
│   └── questionnaire_raw.csv
├── blueprint/      # 清洗蓝图（代理人提供）
│   └── questionnaire_cleaning.md
├── schema/         # 数据结构（系统生成）
│   └── questionnaire.json
├── processor/      # 清洗处理器（系统生成/人工编写）
│   └── questionnaire_cleaner.py
├── dataset/        # 清洗后数据（系统生成）
│   ├── questionnaire_cleaned.csv
│   └── questionnaire_cleaning.zip
└── manifest/       # 交付清单（系统生成）
    └── questionnaire_cleaning.md
```

## 工作流程

### 步骤1：规范准备（系统预设）

**输入**: 无  
**输出**: `spec/base.md`, `spec/questionnaire_cleaning.md`

系统预设数据清洗的基础规范和特定场景规范。

**验证要点**:
- 规范文件存在
- 包含必要的清洗规则说明

### 步骤2：原始数据接收（委托人提供）

**输入**: 委托人提供的数据文件  
**输出**: `record/questionnaire_raw.csv`

接收原始数据，文件格式应为CSV。

**验证要点**:
- 数据文件存在
- 包含必需的列（如：提交时间、年龄、工作年限、所属部门）
- 数据格式符合基本要求

### 步骤3：清洗蓝图定义（代理人提供）

**输入**: 代理人定义的清洗规则  
**输出**: `blueprint/questionnaire_cleaning.md`

定义数据清洗的详细规则，包括字段映射、类型转换、缺失值处理等。

**验证要点**:
- 蓝图文件存在
- 包含必需章节：`## 数据模型`、`## 数据处理流程`
- 包含字段定义表格（字段名、类型、缺失编码等）

### 步骤4：生成Schema和Processor

**输入**: 蓝图 + 规范  
**输出**: `schema/questionnaire.json`, `processor/questionnaire_cleaner.py`

根据蓝图生成数据结构定义和清洗处理器代码。

**验证要点**:
- Processor文件存在
- Processor包含`QuestionnaireCleaner`类
- Processor包含`process`方法
- Schema文件存在（可选）

### 步骤5：执行数据清洗

**输入**: `record/questionnaire_raw.csv` + `processor/questionnaire_cleaner.py`  
**输出**: `dataset/questionnaire_cleaned.csv`

运行处理器对原始数据进行清洗。

**验证要点**:
- 清洗后数据文件存在
- 数据形状符合预期
- 列名与期望一致
- 数据内容与期望一致
- 符合蓝图定义的字段要求

### 步骤6：生成交付清单

**输入**: 所有生成产物  
**输出**: `manifest/questionnaire_cleaning.md`

自动生成交付清单，记录所有交付物和数据流转路径。

**验证要点**:
- 交付清单文件存在
- 包含必需章节：
  - `## 📦 交付物清单`
  - `## 🔄 数据流转路径`
  - `## ✅ 质量验证`
- 提及所有必需交付物：blueprint、spec、processor、record、dataset

### 步骤7：打包数据集（可选）

**输入**: `dataset/questionnaire_cleaned.csv`  
**输出**: `dataset/questionnaire_cleaning.zip`

将清洗后的数据打包，便于分发和部署。

**验证要点**:
- ZIP包包含清洗后的数据文件

## 质量标准

### 数据质量检查

- **必填字段**: 不能为空（如`submit_time`）
- **数值范围**: 符合业务规则（如年龄在18-70范围内）
- **编码规范**: 使用预定义的编码值（如部门编码1-5）
- **缺失值处理**: 统一使用`-99`标记缺失

### 文档质量标准

- **蓝图**: 必须清晰定义每个字段的转换规则
- **Manifest**: 必须完整记录数据流转过程
- **Spec**: 必须提供清晰的清洗规范说明

## 扩展指南

### 新项目启动

1. 在`spec/`目录创建新的规范文件
2. 委托人提供原始数据到`record/`
3. 代理人创建清洗蓝图到`blueprint/`
4. 编写或生成Processor到`processor/`
5. 执行清洗验证流程
6. 生成最终交付物

### 不同数据类型

本工作流程不仅限于问卷数据，可扩展至：
- 客户数据清洗
- 产品数据标准化
- 财务数据整理
- 日志数据处理

只需相应调整：
- 规范文件命名
- 必需列定义
- 清洗规则蓝图
- 处理器实现

## 工具链

### 测试框架

使用pytest进行集成测试，确保每个步骤的正确性。

### 数据处理

- **Pandas**: 数据读取和处理
- **Python**: 处理器编写

### 版本控制

建议使用Git管理所有文件，包括：
- 规范文档
- 蓝图定义
- 测试用例
- 数据样本（小规模）

## 附录

### 示例项目

完整的示例项目位于 `tests/fixtures/questionnaire_cleanning/`，包含：
- 完整的测试数据
- 参考实现
- 集成测试用例

### 相关文档

- `docs/spec/README.md`: 工程标准
- `docs/qa/questionnaire_cleanning.md`: 测试流程说明
- `src/python_sdk/integrated_tests/test_questionnaire_cleanning.py`: 集成测试代码
