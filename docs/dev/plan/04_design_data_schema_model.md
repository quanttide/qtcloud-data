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
