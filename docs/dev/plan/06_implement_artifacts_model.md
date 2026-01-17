# TASK_006: 实现 BusinessArtifacts 业务工件模型

## 上下文

- **阶段**: 阶段一 - 领域模型设计与实现
- **优先级**: P0（必须完成）
- **预计工时**: 1 小时
- **前置任务**: `05_implement_workspace_model.md`
- **后置任务**: `07_implement_data_pipeline_model.md`

## 任务目标

根据设计文档 `02_design_artifacts_model.md` 实现 `BusinessArtifacts` 类。

## 实现步骤

### 1. 实现 BusinessArtifacts 类

创建文件 `packages/quanttide_data_toolkit/src/quanttide_data_toolkit/artifacts.py`：

```python
import json
from pathlib import Path
from typing import Dict, List
from .workspace import Workspace


class BusinessArtifacts:
    """业务工件

    业务工件是数据清洗项目中的关键交付物，包括 Plan、Schema、Manifest 等。

    Attributes:
        workspace (Workspace): 工作区实例
    """

    def __init__(self, workspace: Workspace):
        """初始化业务工件

        Args:
            workspace: 工作区实例
        """
        self.workspace = workspace

    def plan_is_clear(self) -> bool:
        """Plan 清晰度检查

        检查 Plan 是否包含数据模型和数据处理流程章节。

        Returns:
            bool: 如果 Plan 包含所有必需章节，返回 True
        """
        plan_path = self.workspace.get_component("plan")
        plan_file = list(plan_path.glob("*.md"))[0]
        content = plan_file.read_text(encoding='utf-8')

        required_sections = ["## 数据模型", "## 数据处理流程"]
        for section in required_sections:
            if section not in content:
                return False
        return True

    def schema_is_complete(self) -> bool:
        """Schema 完整性检查

        检查 Schema 是否包含所有必需字段和完整的字段定义。

        Returns:
            bool: 如果 Schema 完整，返回 True
        """
        schema_path = self.workspace.get_component("schema")
        schema_file = list(schema_path.glob("*.json"))[0]

        with open(schema_file, encoding='utf-8') as f:
            schema = json.load(f)

        required_fields = ["name", "version", "schema", "quality_rules", "transformations"]
        for field in required_fields:
            if field not in schema:
                return False

        return self.schema_is_well_formed()

    def schema_is_well_formed(self) -> bool:
        """Schema 格式检查

        检查 Schema 是否为有效的 JSON 格式。

        Returns:
            bool: 如果 Schema 格式正确，返回 True
        """
        schema_path = self.workspace.get_component("schema")
        schema_file = list(schema_path.glob("*.json"))[0]

        try:
            with open(schema_file, encoding='utf-8') as f:
                schema = json.load(f)

            if "fields" not in schema.get("schema", {}):
                return False

            fields = schema["schema"]["fields"]
            if not isinstance(fields, list) or len(fields) == 0:
                return False

            valid_types = {
                "string", "integer", "float", "binary",
                "datetime", "categorical", "text"
            }

            for field in fields:
                if "name" not in field or "type" not in field:
                    return False
                if field["type"] not in valid_types:
                    return False

            return True

        except (json.JSONDecodeError, FileNotFoundError):
            return False

    def manifest_is_complete(self) -> bool:
        """Manifest 完整性检查

        检查 Manifest 是否包含所有必需组件和文件引用。

        Returns:
            bool: 如果 Manifest 完整，返回 True
        """
        manifest_path = self.workspace.get_component("manifest")
        manifest_file = list(manifest_path.glob("*.json"))[0]

        try:
            with open(manifest_file, encoding='utf-8') as f:
                manifest = json.load(f)

            required_fields = [
                "order_id", "customer", "project_name",
                "created_at", "status", "includes"
            ]

            for field in required_fields:
                if field not in manifest:
                    return False

            include_types = {item["type"] for item in manifest["includes"]}
            required_types = {"recipe", "dataset", "plan", "schema", "inspector", "report"}

            return include_types == required_types

        except (json.JSONDecodeError, FileNotFoundError):
            return False

    def validation_report(self) -> str:
        """生成验证报告

        Returns:
            str: 详细的验证报告，列出所有检查项的结果
        """
        results = {
            "Plan 清晰度": self.plan_is_clear(),
            "Schema 完整性": self.schema_is_complete(),
            "Schema 格式": self.schema_is_well_formed(),
            "Manifest 完整性": self.manifest_is_complete(),
        }

        report = "业务工件验证报告：\n"
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

__all__ = ['Workspace', 'BusinessArtifacts']
```

### 3. 编写单元测试

创建文件 `packages/quanttide_data_toolkit/tests/quanttide_data_toolkit/test_artifacts.py`：

```python
import pytest
from pathlib import Path
import sys

# 添加 src 到路径
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))

from quanttide_data_toolkit import Workspace, BusinessArtifacts


class TestBusinessArtifacts:
    """BusinessArtifacts 领域模型测试"""

    @pytest.fixture
    def workspace(self):
        workspace_root = Path(__file__).parent.parent.parent.parent.parent / "tests" / "fixtures" / "workspace"
        return Workspace(workspace_root)

    @pytest.fixture
    def artifacts(self, workspace):
        return BusinessArtifacts(workspace)

    def test_plan_is_clear(self, artifacts):
        """Plan 清晰度"""
        assert artifacts.plan_is_clear()

    def test_schema_is_complete(self, artifacts):
        """Schema 完整性"""
        assert artifacts.schema_is_complete()

    def test_schema_is_well_formed(self, artifacts):
        """Schema 格式"""
        assert artifacts.schema_is_well_formed()

    def test_manifest_is_complete(self, artifacts):
        """Manifest 完整性"""
        assert artifacts.manifest_is_complete()

    def test_validation_report(self, artifacts):
        """验证报告"""
        report = artifacts.validation_report()
        assert report is not None
        assert isinstance(report, str)
```

## 交付物

- 实现代码：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/artifacts.py`
- 更新包初始化：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/__init__.py`
- 单元测试：`packages/quanttide_data_toolkit/tests/quanttide_data_toolkit/test_artifacts.py`

## 验证标准

- [ ] 代码实现符合设计文档
- [ ] 所有单元测试通过
- [ ] 代码风格符合项目规范

## 参考

- 设计文档：`docs/dev/plan/02_design_artifacts_model.md`
- 原始测试：`tests/test_fixtures.py` 第 48-114 行
