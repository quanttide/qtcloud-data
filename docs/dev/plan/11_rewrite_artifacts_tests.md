# TASK_011: 重写 BusinessArtifacts 测试

## 上下文

- **阶段**: 阶段二 - 测试重构
- **优先级**: P0（必须完成）
- **预计工时**: 1 小时
- **前置任务**: `10_rewrite_workspace_tests.md`
- **后置任务**: `12_rewrite_data_pipeline_tests.md`

## 任务目标

使用 `quanttide_data_toolkit` 重写 BusinessArtifacts 测试。

## 实施步骤

### 1. 创建测试文件

创建文件 `tests/fixtures/business/test_business_artifacts.py`：

```python
"""业务工件测试

本模块测试业务工件的完整性和质量，使用 quanttide_data_toolkit。
"""

import pytest
from pathlib import Path
from quanttide_data_toolkit import Workspace, BusinessArtifacts

FIXTURES_ROOT = Path(__file__).parent.parent / "workspace"


class TestBusinessArtifacts:
    """业务工件验证

    业务工件是数据清洗项目中的关键交付物。
    """

    @pytest.fixture
    def workspace(self):
        """工作区 fixture"""
        return Workspace(FIXTURES_ROOT)

    @pytest.fixture
    def artifacts(self, workspace):
        """业务工件 fixture"""
        return BusinessArtifacts(workspace)

    def test_plan_clear(self, artifacts):
        """Plan 清晰度"""
        assert artifacts.plan_is_clear()

    def test_schema_complete(self, artifacts):
        """Schema 完整性"""
        assert artifacts.schema_is_complete()

    def test_schema_well_formed(self, artifacts):
        """Schema 格式"""
        assert artifacts.schema_is_well_formed()

    def test_manifest_complete(self, artifacts):
        """Manifest 完整性"""
        assert artifacts.manifest_is_complete()

    def test_artifacts_validation_report(self, artifacts):
        """业务工件验证报告"""
        report = artifacts.validation_report()
        assert report is not None
        assert isinstance(report, str)
```

## 交付物

- 测试文件：`tests/fixtures/business/test_business_artifacts.py`

## 参考

- 领域模型：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/artifacts.py`
- 原始测试：`tests/test_fixtures.py` 第 48-114 行、218-270 行
