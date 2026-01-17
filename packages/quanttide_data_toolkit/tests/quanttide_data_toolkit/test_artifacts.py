import pytest
from pathlib import Path
import sys

# 添加 src 到路径
src_path = str(Path(__file__).parent.parent.parent / "src")
sys.path.insert(0, src_path)

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
        """Plan 清晰度检查通过"""
        assert artifacts.plan_is_clear()

    def test_schema_is_complete(self, artifacts):
        """Schema 完整性检查通过"""
        assert artifacts.schema_is_complete()

    def test_schema_is_well_formed(self, artifacts):
        """Schema 格式检查通过"""
        assert artifacts.schema_is_well_formed()

    def test_manifest_is_complete(self, artifacts):
        """Manifest 完整性检查通过"""
        assert artifacts.manifest_is_complete()

    def test_validation_report(self, artifacts):
        """验证报告生成正常"""
        report = artifacts.validation_report()
        assert report is not None
        assert isinstance(report, str)
