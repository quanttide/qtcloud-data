#!/bin/bash
# 批量生成剩余任务文件

PLAN_DIR="/Users/mac/repos/quanttide/domains/quanttide-data/src/qtcloud-data/docs/dev/plan"
PACKAGE_PREFIX="packages/quanttide_data_toolkit"
PACKAGE_SRC="${PACKAGE_PREFIX}/src/quanttide_data_toolkit"
PACKAGE_TESTS="${PACKAGE_PREFIX}/tests/quanttide_data_toolkit"

# Task 02: Design Artifacts Model
cat > "${PLAN_DIR}/02_design_artifacts_model.md" << 'EOF'
# TASK_002: 设计 BusinessArtifacts 业务工件模型

## 上下文

- **阶段**: 阶段一 - 领域模型设计与实现
- **优先级**: P0（必须完成）
- **预计工时**: 0.5 小时
- **前置任务**: `01_design_workspace_model.md`
- **后置任务**: `03_design_data_pipeline_model.md`

## 业务目标

创建 `BusinessArtifacts` 领域模型，封装业务工件验证的业务逻辑。

## 设计要求

### 类定义

```python
class BusinessArtifacts:
    """业务工件

    业务工件是数据清洗项目中的关键交付物，包括 Plan、Schema、Manifest 等。
    """

    def __init__(self, workspace: Workspace):
        """初始化业务工件"""
        pass

    def plan_is_clear(self) -> bool:
        """Plan 清晰度检查"""
        pass

    def schema_is_complete(self) -> bool:
        """Schema 完整性检查"""
        pass

    def schema_is_well_formed(self) -> bool:
        """Schema 格式检查"""
        pass

    def manifest_is_complete(self) -> bool:
        """Manifest 完整性检查"""
        pass

    def validation_report(self) -> str:
        """生成验证报告"""
        pass
```

## 交付物

- 设计文档：`packages/quanttide_data_toolkit/docs/design/artifacts.md`

## 验证标准

- [ ] 类定义符合业务语义
- [ ] 方法命名使用业务术语
- [ ] 封装了所有技术细节

## 参考

- 原始测试：`tests/test_fixtures.py` 第 48-114 行
EOF

echo "Generated 02_design_artifacts_model.md"

# Task 03: Design DataPipeline Model
cat > "${PLAN_DIR}/03_design_data_pipeline_model.md" << 'EOF'
# TASK_003: 设计 DataPipeline 数据流水线模型

## 上下文

- **阶段**: 阶段一 - 领域模型设计与实现
- **优先级**: P0（必须完成）
- **预计工时**: 0.5 小时
- **前置任务**: `02_design_artifacts_model.md`
- **后置任务**: `04_design_data_schema_model.md`

## 业务目标

创建 `DataPipeline` 领域模型，封装数据流水线验证的业务逻辑。

## 设计要求

### 类定义

```python
class DataPipeline:
    """数据流水线

    数据流水线是数据清洗和验证的完整流程。
    """

    def __init__(self, workspace: Workspace):
        """初始化数据流水线"""
        pass

    def inspector_available(self) -> bool:
        """Inspector 可用性检查"""
        pass

    def inspector_complies_with_schema(self) -> bool:
        """Inspector Schema 合规性检查"""
        pass

    def data_quality_acceptable(self) -> bool:
        """数据质量可接受性检查"""
        pass

    def business_rules_complied(self) -> bool:
        """业务规则合规性检查"""
        pass

    def validation_report(self) -> str:
        """生成验证报告"""
        pass
```

## 交付物

- 设计文档：`packages/quanttide_data_toolkit/docs/design/data_pipeline.md`

## 验证标准

- [ ] 类定义符合业务语义
- [ ] 方法命名使用业务术语

## 参考

- 原始测试：`tests/test_fixtures.py` 第 165-216 行
EOF

echo "Generated 03_design_data_pipeline_model.md"

# Task 04: Design DataSchema Model
cat > "${PLAN_DIR}/04_design_data_schema_model.md" << 'EOF'
# TASK_004: 设计 DataSchema 数据结构模型

## 上下文

- **阶段**: 阶段一 - 领域模型设计与实现
- **优先级**: P0（必须完成）
- **预计工时**: 0.5 小时
- **前置任务**: `03_design_data_pipeline_model.md`
- **后置任务**: `05_implement_workspace_model.md`

## 业务目标

创建 `DataSchema` 领域模型，封装数据结构验证的业务逻辑。

## 设计要求

### 类定义

```python
class DataSchema:
    """数据结构定义"""

    VALID_TYPES = {
        "string", "integer", "float", "binary",
        "datetime", "categorical", "text"
    }

    def __init__(self, schema_path: Path):
        """初始化数据结构定义"""
        pass

    def is_complete(self) -> bool:
        """Schema 完整性检查"""
        pass

    def is_well_defined(self) -> bool:
        """Schema 定义检查"""
        pass

    def fields_match(self, data_columns: set) -> bool:
        """字段匹配检查"""
        pass

    def validation_report(self) -> str:
        """生成验证报告"""
        pass
```

## 交付物

- 设计文档：`packages/quanttide_data_toolkit/docs/design/data_schema.md`

## 验证标准

- [ ] 类定义符合业务语义

## 参考

- 原始测试：`tests/test_fixtures.py` 第 98-114 行
EOF

echo "Generated 04_design_data_schema_model.md"

echo ""
echo "Task files generated successfully!"
echo "Total: 4 design tasks (01-04)"
