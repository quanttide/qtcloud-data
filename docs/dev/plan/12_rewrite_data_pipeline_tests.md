# TASK_012: 重写 DataPipeline 测试

## 上下文

- **阶段**: 阶段二 - 测试重构
- **优先级**: P0（必须完成）
- **预计工时**: 1 小时
- **前置任务**: `11_rewrite_artifacts_tests.md`
- **后置任务**: `13_rewrite_schema_tests.md`

## 任务目标

使用 `quanttide_data_toolkit` 重写 DataPipeline 测试。

## 实施步骤

### 1. 创建测试文件

创建文件 `tests/fixtures/business/test_data_pipeline.py`：

```python
"""数据流水线测试

本模块测试数据流水线的完整性和质量，使用 quanttide_data_toolkit。
"""

import pytest
from pathlib import Path
from quanttide_data_toolkit import Workspace, DataPipeline

FIXTURES_ROOT = Path(__file__).parent.parent / "workspace"


class TestDataPipeline:
    """数据流水线验证

    数据流水线是数据清洗和验证的完整流程。
    """

    @pytest.fixture
    def workspace(self):
        """工作区 fixture"""
        return Workspace(FIXTURES_ROOT)

    @pytest.fixture
    def pipeline(self, workspace):
        """数据流水线 fixture"""
        return DataPipeline(workspace)

    def test_inspector_available(self, pipeline):
        """Inspector 可用性"""
        assert pipeline.inspector_available()

    def test_inspector_complies_with_schema(self, pipeline):
        """Inspector Schema 合规性"""
        assert pipeline.inspector_complies_with_schema()

    def test_data_quality_acceptable(self, pipeline):
        """数据质量可接受性"""
        assert pipeline.data_quality_acceptable()

    def test_business_rules_complied(self, pipeline):
        """业务规则合规性"""
        assert pipeline.business_rules_complied()

    def test_pipeline_validation_report(self, pipeline):
        """数据流水线验证报告"""
        report = pipeline.validation_report()
        assert report is not None
        assert isinstance(report, str)
```

## 交付物

- 测试文件：`tests/fixtures/business/test_data_pipeline.py`

## 参考

- 领域模型：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/data_pipeline.py`
- 原始测试：`tests/test_fixtures.py` 第 165-216 行
