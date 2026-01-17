# TASK_007: 实现 DataPipeline 数据流水线模型

## 上下文

- **阶段**: 阶段一 - 领域模型设计与实现
- **优先级**: P0（必须完成）
- **预计工时**: 1 小时
- **前置任务**: `06_implement_artifacts_model.md`
- **后置任务**: `08_implement_data_schema_model.md`

## 任务目标

根据设计文档 `03_design_data_pipeline_model.md` 实现 `DataPipeline` 类。

## 实现步骤

### 1. 实现 DataPipeline 类

创建文件 `packages/quanttide_data_toolkit/src/quanttide_data_toolkit/data_pipeline.py`：

```python
import pandas as pd
from pathlib import Path
from typing import Dict, Any
from .workspace import Workspace

# 导入现有 Inspector（临时方案，未来封装为适配器）
from fixtures.workspace.inspector.questionnaire_inspector import QuestionnaireInspector


class DataPipeline:
    """数据流水线

    数据流水线是数据清洗和验证的完整流程，包括 Processor、Inspector 等。

    Attributes:
        workspace (Workspace): 工作区实例
    """

    def __init__(self, workspace: Workspace):
        """初始化数据流水线

        Args:
            workspace: 工作区实例
        """
        self.workspace = workspace

    def _get_inspector(self) -> QuestionnaireInspector:
        """获取 Inspector 实例

        Returns:
            QuestionnaireInspector: Inspector 实例
        """
        plan_path = self.workspace.get_component("plan")
        plan_file = list(plan_path.glob("*.md"))[0]
        return QuestionnaireInspector(plan_file)

    def _get_cleaned_data(self) -> pd.DataFrame:
        """获取清洗后的数据

        Returns:
            pd.DataFrame: 清洗后的数据
        """
        record_path = self.workspace.get_component("record")
        data_file = record_path / "questionnaire_cleaned.csv"
        return pd.read_csv(data_file)

    def inspector_available(self) -> bool:
        """Inspector 可用性检查

        检查 Inspector 是否可以初始化并运行。

        Returns:
            bool: 如果 Inspector 可用，返回 True
        """
        try:
            inspector = self._get_inspector()
            return inspector is not None and len(inspector.field_definitions) > 0
        except Exception:
            return False

    def inspector_complies_with_schema(self) -> bool:
        """Inspector Schema 合规性检查

        检查 Inspector 的验证结果是否符合 Schema 定义。

        Returns:
            bool: 如果合规，返回 True
        """
        try:
            inspector = self._get_inspector()
            data = self._get_cleaned_data()
            result = inspector.validate_schema_compliance(data)

            return (
                result is not None
                and "status" in result
                and "issues" in result
                and isinstance(result["issues"], list)
            )
        except Exception:
            return False

    def data_quality_acceptable(self) -> bool:
        """数据质量可接受性检查

        检查数据质量检查结果是否可接受。

        Returns:
            bool: 如果数据质量可接受，返回 True
        """
        try:
            inspector = self._get_inspector()
            data = self._get_cleaned_data()
            result = inspector.validate_data_quality(data)

            return (
                result is not None
                and "status" in result
                and "checks" in result
            )
        except Exception:
            return False

    def business_rules_complied(self) -> bool:
        """业务规则合规性检查

        检查数据是否符合所有业务规则。

        Returns:
            bool: 如果符合，返回 True
        """
        try:
            inspector = self._get_inspector()
            data = self._get_cleaned_data()
            result = inspector.validate_business_rules(data)

            return (
                result is not None
                and "status" in result
                and "issues" in result
            )
        except Exception:
            return False

    def validation_report(self) -> str:
        """生成验证报告

        Returns:
            str: 详细的验证报告，列出所有检查项的结果
        """
        results = {
            "Inspector 可用性": self.inspector_available(),
            "Inspector Schema 合规性": self.inspector_complies_with_schema(),
            "数据质量可接受性": self.data_quality_acceptable(),
            "业务规则合规性": self.business_rules_complied(),
        }

        report = "数据流水线验证报告：\n"
        for check, passed in results.items():
            status = "✅" if passed else "❌"
            report += f"  {status} {check}\n"

        return report
```

### 2. 更新包 __init__.py

更新文件 `packages/quanttide_data_toolkit/src/quanttide_data_toolkit/__init__.py`：

```python
from .workspace import Workspace
from .artifacts import BusinessArtifacts
from .data_pipeline import DataPipeline

__all__ = ['Workspace', 'BusinessArtifacts', 'DataPipeline']
```

### 3. 编写单元测试

创建文件 `packages/quanttide_data_toolkit/tests/quanttide_data_toolkit/test_data_pipeline.py`：

```python
import pytest
from pathlib import Path
import sys

# 添加 src 到路径
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))

from quanttide_data_toolkit import Workspace, DataPipeline


class TestDataPipeline:
    """DataPipeline 领域模型测试"""

    @pytest.fixture
    def workspace(self):
        workspace_root = Path(__file__).parent.parent.parent.parent.parent / "tests" / "fixtures" / "workspace"
        return Workspace(workspace_root)

    @pytest.fixture
    def pipeline(self, workspace):
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

    def test_validation_report(self, pipeline):
        """验证报告"""
        report = pipeline.validation_report()
        assert report is not None
        assert isinstance(report, str)
```

## 交付物

- 实现代码：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/data_pipeline.py`
- 更新包初始化：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/__init__.py`
- 单元测试：`packages/quanttide_data_toolkit/tests/quanttide_data_toolkit/test_data_pipeline.py`

## 验证标准

- [ ] 代码实现符合设计文档
- [ ] 所有单元测试通过
- [ ] 代码风格符合项目规范

## 参考

- 设计文档：`docs/dev/plan/03_design_data_pipeline_model.md`
- 原始测试：`tests/test_fixtures.py` 第 165-216 行
