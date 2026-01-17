# Test Fixtures 重构计划 - 任务索引

本目录包含重构计划的所有任务文件，按实施顺序排列。

## 项目结构

```
src/qtcloud-data/                       # 当前项目
├── src/                               # 平台层代码
│   ├── cli/
│   ├── provider/
│   ├── python_sdk/
│   └── studio/
│
└── packages/                          # 可复用包
    └── quanttide_data_toolkit/       # 领域模型库（本次重点）
        ├── src/
        │   └── quanttide_data_toolkit/
        │       ├── __init__.py
        │       ├── workspace.py
        │       ├── artifacts.py
        │       ├── data_schema.py
        │       └── data_pipeline.py
        └── tests/
            └── quanttide_data_toolkit/
                ├── test_workspace.py
                ├── test_artifacts.py
                ├── test_data_schema.py
                └── test_data_pipeline.py
```

## 任务执行顺序

### 阶段一：领域模型设计与实现（P0）
- `01_design_workspace_model.md` - 设计 Workspace 工作区模型（0.5h）
- `02_design_artifacts_model.md` - 设计 BusinessArtifacts 业务工件模型（0.5h）
- `03_design_data_pipeline_model.md` - 设计 DataPipeline 数据流水线模型（0.5h）
- `04_design_data_schema_model.md` - 设计 DataSchema 数据结构模型（0.5h）
- `05_implement_workspace_model.md` - 实现 Workspace 类（1h）
- `06_implement_artifacts_model.md` - 实现 BusinessArtifacts 类（1h）
- `07_implement_data_pipeline_model.md` - 实现 DataPipeline 类（1h）
- `08_implement_data_schema_model.md` - 实现 DataSchema 类（1h）

### 阶段二：测试重构（P0）
- `09_refactor_test_structure.md` - 重构测试结构（2h）
- `10_rewrite_workspace_tests.md` - 重写 Workspace 测试（1h）
- `11_rewrite_artifacts_tests.md` - 重写 BusinessArtifacts 测试（1h）
- `12_rewrite_data_pipeline_tests.md` - 重写 DataPipeline 测试（1h）
- `13_rewrite_schema_tests.md` - 重写 DataSchema 和 Report 测试（1h）

### 阶段三：术语对齐与文档完善（P1）
- `14_create_glossary.md` - 创建业务术语词典（1h）
- `15_replace_technical_terms.md` - 替换技术术语（1h）
- `16_improve_documentation.md` - 完善文档（1h）

### 阶段四：验证与优化（P1）
- `17_qa_reaudit.md` - QA 重新审计（1h）
- `18_business_review.md` - 业务人员评审（1h）

## 快速开始

1. 按顺序执行 `01` 到 `08` 号任务（阶段一）
2. 按顺序执行 `09` 到 `13` 号任务（阶段二）
3. 按顺序执行 `14` 到 `16` 号任务（阶段三，可选）
4. 按顺序执行 `17` 到 `18` 号任务（阶段四，可选）

## 依赖关系

- 阶段二依赖阶段一（必须先完成领域模型）
- 阶段三依赖阶段二（必须先完成测试重构）
- 阶段四依赖阶段三（必须先完成术语对齐）

## 总工时

- P0 任务：12 小时（必须完成）
- P1 任务：5 小时（建议完成）
- 总计：17 小时

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
