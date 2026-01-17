import pytest
from pathlib import Path
import sys

# 添加 src 到路径
src_path = str(Path(__file__).parent.parent.parent / "src")
sys.path.insert(0, src_path)

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
        """Inspector 可用性检查通过"""
        assert pipeline.inspector_available()

    def test_inspector_complies_with_schema(self, pipeline):
        """Inspector Schema 合规性检查通过"""
        assert pipeline.inspector_complies_with_schema()

    def test_data_quality_acceptable(self, pipeline):
        """数据质量可接受性检查通过"""
        assert pipeline.data_quality_acceptable()

    def test_business_rules_complied(self, pipeline):
        """业务规则合规性检查通过"""
        assert pipeline.business_rules_complied()

    def test_validation_report(self, pipeline):
        """验证报告生成正常"""
        report = pipeline.validation_report()
        assert report is not None
        assert isinstance(report, str)
