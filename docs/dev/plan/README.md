# Test Fixtures 重构计划 - 完整任务列表

## 概述

本计划将 `tests/test_fixtures.py` 从技术视角重构为业务视角，创建 `quanttide_data_toolkit` 领域模型库。

## 项目结构

```
src/qtcloud-data/
├── src/                          # 平台层代码
│   ├── cli/
│   ├── provider/
│   ├── python_sdk/
│   └── studio/
│
└── packages/                     # 可复用包
    └── quanttide_data_toolkit/   # 领域模型库（本次重点）
        ├── src/quanttide_data_toolkit/
        │   ├── __init__.py
        │   ├── workspace.py
        │   ├── artifacts.py
        │   ├── data_pipeline.py
        │   └── data_schema.py
        └── tests/quanttide_data_toolkit/
            ├── test_workspace.py
            ├── test_artifacts.py
            ├── test_data_pipeline.py
            └── test_data_schema.py
```

## 任务列表

### 阶段一：领域模型设计与实现（P0）- 6 小时

| 任务ID | 任务名称 | 文件 | 工时 |
|--------|---------|------|------|
| TASK_001 | 设计 Workspace 模型 | 01_design_workspace_model.md | 0.5h |
| TASK_002 | 设计 BusinessArtifacts 模型 | 02_design_artifacts_model.md | 0.5h |
| TASK_003 | 设计 DataPipeline 模型 | 03_design_data_pipeline_model.md | 0.5h |
| TASK_004 | 设计 DataSchema 模型 | 04_design_data_schema_model.md | 0.5h |
| TASK_005 | 实现 Workspace 类 | 05_implement_workspace_model.md | 1h |
| TASK_006 | 实现 BusinessArtifacts 类 | 06_implement_artifacts_model.md | 1h |
| TASK_007 | 实现 DataPipeline 类 | 07_implement_data_pipeline_model.md | 1h |
| TASK_008 | 实现 DataSchema 类 | 08_implement_data_schema_model.md | 1h |

### 阶段二：测试重构（P0）- 6 小时

| 任务ID | 任务名称 | 文件 | 工时 |
|--------|---------|------|------|
| TASK_009 | 重构测试结构 | 09_refactor_test_structure.md | 2h |
| TASK_010 | 重写 Workspace 测试 | 10_rewrite_workspace_tests.md | 1h |
| TASK_011 | 重写 BusinessArtifacts 测试 | 11_rewrite_artifacts_tests.md | 1h |
| TASK_012 | 重写 DataPipeline 测试 | 12_rewrite_data_pipeline_tests.md | 1h |
| TASK_013 | 重写 DataSchema 和 Report 测试 | 13_rewrite_schema_tests.md | 1h |

### 阶段三：术语对齐与文档完善（P1）- 3 小时

| 任务ID | 任务名称 | 文件 | 工时 |
|--------|---------|------|------|
| TASK_014 | 创建业务术语词典 | 14_create_glossary.md | 1h |
| TASK_015 | 替换技术术语 | 15_replace_technical_terms.md | 1h |
| TASK_016 | 完善文档 | 16_improve_documentation.md | 1h |

### 阶段四：验证与优化（P1）- 2 小时

| 任务ID | 任务名称 | 文件 | 工时 |
|--------|---------|------|------|
| TASK_017 | QA 重新审计 | 17_qa_reaudit.md | 1h |
| TASK_018 | 业务人员评审 | 18_business_review.md | 1h |

## 总工时

- P0 任务：12 小时（必须完成）
- P1 任务：5 小时（建议完成）
- 总计：17 小时

## 依赖关系

```
TASK_001 → TASK_002 → TASK_003 → TASK_004
    ↓         ↓         ↓         ↓
TASK_005 → TASK_006 → TASK_007 → TASK_008
                                ↓
                         TASK_009 → TASK_010 → TASK_011 → TASK_012 → TASK_013
                                                                ↓
                                                   TASK_014 → TASK_015 → TASK_016
                                                                   ↓
                                                   TASK_017 → TASK_018
```

## 快速开始

### 方式 1：按顺序执行

```bash
# 依次执行 01-18 号任务
cd docs/dev/plan
# 阅读 00_index.md 了解整体计划
# 然后按顺序执行每个任务文件
```

### 方式 2：按阶段执行

```bash
# 阶段一：领域模型设计与实现
# 执行 TASK_001 ~ TASK_008

# 阶段二：测试重构
# 执行 TASK_009 ~ TASK_013

# 阶段三：术语对齐与文档完善（可选）
# 执行 TASK_014 ~ TASK_016

# 阶段四：验证与优化（可选）
# 执行 TASK_017 ~ TASK_018
```

## 成功指标

| 指标 | 当前值 | 目标值 |
|------|--------|--------|
| 业务意图表达评分 | 12/50 (24%) | 40/50 (80%) |
| 测试通过率 | 100% | 100% |
| 业务可读性评分 | 2/10 | 8/10 |

## 未来迁移

当前在 `packages/quanttide_data_toolkit/` 开发，未来将迁移为独立仓库：

```bash
# 迁移为独立仓库
git subtree push --prefix=src/qtcloud-data/packages/quanttide_data_toolkit \
    https://github.com/quanttide/quanttide-data-toolkit.git main

# 替换为 submodule
git submodule add https://github.com/quanttide/quanttide-data-toolkit.git \
    src/qtcloud-data/packages/quanttide_data_toolkit
```

## 里程碑

| 里程碑 | 日期 | 交付物 |
|--------|------|--------|
| M1: 领域模型完成 | 2026-01-17 | quanttide_data_toolkit 领域模型代码和测试 |
| M2: 测试重构完成 | 2026-01-18 | 重构后的测试代码 |
| M3: 术语对齐完成 | 2026-01-19 | 术语词典和更新文档 |
| M4: 验收通过 | 2026-01-19 | QA 审计报告和业务评审反馈 |

## 参考资料

- QA 审计报告：`docs/qa/report/test_fixtures.md`
- 业务意图清晰度规则：`docs/qa/criteria/business_intent_clarity.md`
- 原始测试：`tests/test_fixtures.py`
