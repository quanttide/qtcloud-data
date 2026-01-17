import pytest
from pathlib import Path
import sys

# 添加 src 到路径
src_path = str(Path(__file__).parent.parent.parent / "src")
sys.path.insert(0, src_path)

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
