# TASK_005: 实现 Workspace 工作区模型

## 上下文

- **阶段**: 阶段一 - 领域模型设计与实现
- **优先级**: P0（必须完成）
- **预计工时**: 1 小时
- **前置任务**: `04_design_data_schema_model.md`
- **后置任务**: `06_implement_artifacts_model.md`

## 任务目标

根据设计文档 `01_design_workspace_model.md` 实现 `Workspace` 类。

## 实现步骤

### 1. 创建目录结构

```bash
mkdir -p packages/quanttide_data_toolkit/src/quanttide_data_toolkit
mkdir -p packages/quanttide_data_toolkit/tests/quanttide_data_toolkit
```

### 2. 实现 Workspace 类

创建文件 `packages/quanttide_data_toolkit/src/quanttide_data_toolkit/workspace.py`：

```python
from pathlib import Path
from typing import List


class Workspace:
    """工作区

    工作区是数据清洗项目的完整环境，包含所有必需的组件。

    Attributes:
        root_path (Path): 工作区根目录路径
    """

    REQUIRED_COMPONENTS = [
        "plan",       # 业务意图文件
        "spec",       # 规格说明
        "schema",     # 数据结构定义
        "processor",  # 数据处理器
        "inspector",  # 数据检查器
        "record",     # 数据记录
        "report",     # 质量报告
        "manifest",   # 交付物清单
    ]

    def __init__(self, root_path: Path):
        """初始化工作区

        Args:
            root_path: 工作区根目录路径
        """
        self.root_path = Path(root_path)

    def is_complete(self) -> bool:
        """工作区是否完整

        Returns:
            bool: 如果所有必需组件都存在且为目录，返回 True
        """
        missing_components = self._get_missing_components()
        return len(missing_components) == 0

    def _get_missing_components(self) -> List[str]:
        """获取缺失的组件

        Returns:
            缺失组件名称列表
        """
        missing = []
        for component in self.REQUIRED_COMPONENTS:
            component_path = self.root_path / component
            if not (component_path.exists() and component_path.is_dir()):
                missing.append(component)
        return missing

    def validation_report(self) -> str:
        """生成验证报告

        Returns:
            str: 详细的验证报告，列出缺失或不合法的组件
        """
        missing = self._get_missing_components()

        if len(missing) == 0:
            return "✅ 工作区完整，所有必需组件都存在"

        report = "❌ 工作区不完整，缺失以下组件：\n"
        for component in missing:
            report += f"  - {component}\n"
        return report

    def get_component(self, component_name: str) -> Path:
        """获取组件路径

        Args:
            component_name: 组件名称

        Returns:
            Path: 组件的完整路径

        Raises:
            ValueError: 如果组件名称不合法
        """
        if component_name not in self.REQUIRED_COMPONENTS:
            raise ValueError(f"不合法的组件名称: {component_name}")

        return self.root_path / component_name
```

### 3. 创建包 __init__.py

创建文件 `packages/quanttide_data_toolkit/src/quanttide_data_toolkit/__init__.py`：

```python
from .workspace import Workspace

__all__ = ['Workspace']
```

### 4. 编写单元测试

创建文件 `packages/quanttide_data_toolkit/tests/quanttide_data_toolkit/test_workspace.py`：

```python
import pytest
from pathlib import Path
import sys

# 添加 src 到路径
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))

from quanttide_data_toolkit import Workspace


class TestWorkspace:
    """Workspace 领域模型测试"""

    @pytest.fixture
    def workspace(self):
        # 使用相对路径到 fixtures
        workspace_root = Path(__file__).parent.parent.parent.parent.parent / "tests" / "fixtures" / "workspace"
        return Workspace(workspace_root)

    def test_workspace_complete(self, workspace):
        """工作区完整"""
        assert workspace.is_complete(), workspace.validation_report()

    def test_validation_report(self, workspace):
        """验证报告"""
        report = workspace.validation_report()
        assert report is not None
        assert isinstance(report, str)

    def test_get_component(self, workspace):
        """获取组件路径"""
        plan_path = workspace.get_component("plan")
        assert plan_path.exists()

    def test_get_invalid_component(self, workspace):
        """获取不合法的组件"""
        with pytest.raises(ValueError):
            workspace.get_component("invalid")
```

## 交付物

- 实现代码：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/workspace.py`
- 包初始化：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/__init__.py`
- 单元测试：`packages/quanttide_data_toolkit/tests/quanttide_data_toolkit/test_workspace.py`

## 验证标准

- [ ] 代码实现符合设计文档
- [ ] 所有单元测试通过
- [ ] 代码风格符合项目规范

## 参考

- 设计文档：`docs/dev/plan/01_design_workspace_model.md`
- 原始测试：`tests/test_fixtures.py` 第 24-45 行
