import pytest
from pathlib import Path
import sys
import pandas as pd

# 添加 src 到路径
src_path = str(Path(__file__).parent.parent.parent / "src")
sys.path.insert(0, src_path)

from quanttide_data_toolkit import Workspace, DataSchema


class TestDataSchema:
    """DataSchema 领域模型测试"""

    @pytest.fixture
    def workspace(self):
        workspace_root = Path(__file__).parent.parent.parent.parent.parent / "tests" / "fixtures" / "workspace"
        return Workspace(workspace_root)

    @pytest.fixture
    def schema(self, workspace):
        schema_path = workspace.get_component("schema") / "questionnaire_schema.json"
        return DataSchema(schema_path)

    @pytest.fixture
    def cleaned_data(self, workspace):
        cleaned_path = workspace.get_component("record") / "questionnaire_cleaned.csv"
        return pd.read_csv(cleaned_path)

    def test_is_complete(self, schema):
        """Schema 完整性检查通过"""
        assert schema.is_complete()

    def test_is_well_defined(self, schema):
        """Schema 定义检查通过"""
        assert schema.is_well_defined()

    def test_fields_match(self, schema, cleaned_data):
        """字段匹配检查通过"""
        data_columns = set(cleaned_data.columns)
        assert schema.fields_match(data_columns)

    def test_validation_report(self, schema):
        """验证报告生成正常"""
        report = schema.validation_report()
        assert report is not None
        assert isinstance(report, str)
