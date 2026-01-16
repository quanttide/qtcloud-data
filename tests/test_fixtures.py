"""
问卷清洗 Fixtures 验证单元测试

本模块用于验证 fixtures 目录下的所有组件是否符合规范，包括：
- Plan 存在性与格式
- Schema 结构完整性
- Processor 可执行性
- Inspector 功能正确性
- 数据质量
- Manifest 清单准确性
"""

import pytest
import json
import pandas as pd
from pathlib import Path

from fixtures.workspace.inspector.questionnaire_inspector import QuestionnaireInspector


FIXTURES_ROOT = Path(__file__).parent / "fixtures" / "workspace"


class TestFixturesStructure:
    """验证 fixtures 目录结构完整性"""

    def test_workspace_exists(self):
        """工作区目录存在"""
        assert FIXTURES_ROOT.exists(), f"工作区目录不存在: {FIXTURES_ROOT}"

    def test_required_subdirectories_exist(self):
        """必需的子目录都存在"""
        required_dirs = [
            "plan",
            "spec",
            "schema",
            "processor",
            "inspector",
            "record",
            "report",
            "manifest",
        ]
        for dir_name in required_dirs:
            dir_path = FIXTURES_ROOT / dir_name
            assert dir_path.exists() and dir_path.is_dir(), f"缺少必需目录: {dir_name}"


class TestPlan:
    """验证 Plan（业务意图）文件"""

    @pytest.fixture
    def plan_path(self):
        return FIXTURES_ROOT / "plan" / "questionnaire_cleaning_plan.md"

    def test_plan_exists(self, plan_path):
        """Plan 文件存在"""
        assert plan_path.exists(), f"Plan 文件不存在: {plan_path}"

    def test_plan_has_data_model_section(self, plan_path):
        """Plan 包含数据模型章节"""
        content = plan_path.read_text(encoding='utf-8')
        assert "## 数据模型" in content, "Plan 缺少数据模型章节"

    def test_plan_has_processing_flow_section(self, plan_path):
        """Plan 包含数据处理流程章节"""
        content = plan_path.read_text(encoding='utf-8')
        assert "## 数据处理流程" in content, "Plan 缺少数据处理流程章节"


class TestSchema:
    """验证 Schema 文件"""

    @pytest.fixture
    def schema_path(self):
        return FIXTURES_ROOT / "schema" / "questionnaire_schema.json"

    def test_schema_exists(self, schema_path):
        """Schema 文件存在"""
        assert schema_path.exists(), f"Schema 文件不存在: {schema_path}"

    def test_schema_is_valid_json(self, schema_path):
        """Schema 是有效的 JSON"""
        with open(schema_path, encoding='utf-8') as f:
            data = json.load(f)
        assert data is not None

    @pytest.fixture
    def schema_data(self, schema_path):
        with open(schema_path, encoding='utf-8') as f:
            return json.load(f)

    def test_schema_has_required_fields(self, schema_data):
        """Schema 包含必需字段"""
        required_fields = ["name", "version", "schema", "quality_rules", "transformations"]
        for field in required_fields:
            assert field in schema_data, f"Schema 缺少必需字段: {field}"

    def test_schema_fields_structure(self, schema_data):
        """Schema 字段结构完整"""
        assert "fields" in schema_data["schema"], "Schema 缺少 fields 定义"
        assert isinstance(schema_data["schema"]["fields"], list), "Schema.fields 应该是列表"
        assert len(schema_data["schema"]["fields"]) > 0, "Schema.fields 不能为空"

    def test_schema_field_definitions_complete(self, schema_data):
        """字段定义包含必需属性"""
        valid_types = {"string", "integer", "float", "binary", "datetime", "categorical", "text"}
        for field in schema_data["schema"]["fields"]:
            required_attrs = ["name", "type"]
            for attr in required_attrs:
                assert attr in field, f"字段 {field.get('name')} 缺少必需属性: {attr}"
            # 验证 type 是否合法
            assert field["type"] in valid_types, \
                f"字段 {field['name']} 的 type 不合法: {field['type']}"


class TestDataRecords:
    """验证数据记录文件"""

    @pytest.fixture
    def raw_data_path(self):
        return FIXTURES_ROOT / "record" / "questionnaire_raw.csv"

    @pytest.fixture
    def cleaned_data_path(self):
        return FIXTURES_ROOT / "record" / "questionnaire_cleaned.csv"

    def test_raw_data_exists(self, raw_data_path):
        """原始数据文件存在"""
        assert raw_data_path.exists(), f"原始数据文件不存在: {raw_data_path}"

    def test_cleaned_data_exists(self, cleaned_data_path):
        """清洗后数据文件存在"""
        assert cleaned_data_path.exists(), f"清洗后数据文件不存在: {cleaned_data_path}"

    @pytest.fixture
    def cleaned_df(self, cleaned_data_path):
        return pd.read_csv(cleaned_data_path)

    def test_cleaned_data_not_empty(self, cleaned_df):
        """清洗后数据不为空"""
        assert len(cleaned_df) > 0, "清洗后数据为空"

    @pytest.fixture
    def schema_path(self):
        return FIXTURES_ROOT / "schema" / "questionnaire_schema.json"

    def test_cleaned_data_has_all_schema_fields(self, cleaned_df, schema_path):
        """清洗后数据包含 Schema 中定义的所有字段"""
        with open(schema_path, encoding='utf-8') as f:
            schema = json.load(f)
        schema_fields = {field["name"] for field in schema["schema"]["fields"]}
        data_fields = set(cleaned_df.columns)
        assert schema_fields == data_fields, \
            f"数据字段不匹配: 缺少 {schema_fields - data_fields}, 多余 {data_fields - schema_fields}"

    def test_cleaned_data_time_format(self, cleaned_df):
        """时间字段格式正确"""
        assert "submit_time" in cleaned_df.columns, "缺少 submit_time 字段"
        # 过滤缺失值后再验证时间格式
        time_col = cleaned_df["submit_time"]
        valid_times = time_col[~time_col.isin([-99, "", None, "nan"]) & time_col.notna()]
        pd.to_datetime(valid_times, format="%Y-%m-%d %H:%M:%S", errors="raise")


class TestInspector:
    """验证 Inspector 功能"""

    @pytest.fixture
    def inspector(self):
        plan_path = FIXTURES_ROOT / "plan" / "questionnaire_cleaning_plan.md"
        return QuestionnaireInspector(plan_path)

    def test_inspector_initialization(self, inspector):
        """Inspector 初始化成功"""
        assert inspector is not None
        assert inspector.field_definitions is not None
        assert len(inspector.field_definitions) > 0

    def test_inspector_parse_field_definitions(self, inspector):
        """字段定义解析正确"""
        required_fields = {"age", "tenure_years", "department", "satisfaction", "workload"}
        parsed_fields = {f["name"] for f in inspector.field_definitions}
        assert required_fields.issubset(parsed_fields), \
            f"字段解析不完整: 缺少 {required_fields - parsed_fields}"

    def test_inspector_validate_schema_compliance(self, inspector):
        """Schema 合规性验证"""
        cleaned_path = FIXTURES_ROOT / "record" / "questionnaire_cleaned.csv"
        data = pd.read_csv(cleaned_path)
        result = inspector.validate_schema_compliance(data)

        assert result is not None, "验证结果为空"
        assert "status" in result, "验证结果缺少 status 字段"
        assert "issues" in result, "验证结果缺少 issues 字段"
        assert isinstance(result["issues"], list), "issues 应该是列表"

    def test_inspector_validate_data_quality(self, inspector):
        """数据质量验证"""
        cleaned_path = FIXTURES_ROOT / "record" / "questionnaire_cleaned.csv"
        data = pd.read_csv(cleaned_path)
        result = inspector.validate_data_quality(data)

        assert result is not None, "验证结果为空"
        assert "status" in result, "验证结果缺少 status 字段"
        assert "checks" in result, "验证结果缺少 checks 字段"

    def test_inspector_validate_business_rules(self, inspector):
        """业务规则验证"""
        cleaned_path = FIXTURES_ROOT / "record" / "questionnaire_cleaned.csv"
        data = pd.read_csv(cleaned_path)
        result = inspector.validate_business_rules(data)

        assert result is not None, "验证结果为空"
        assert "status" in result, "验证结果缺少 status 字段"
        assert "issues" in result, "验证结果缺少 issues 字段"


class TestManifest:
    """验证 Manifest 清单文件"""

    @pytest.fixture
    def manifest_path(self):
        return FIXTURES_ROOT / "manifest" / "questionnaire_cleaning_manifest.json"

    def test_manifest_exists(self, manifest_path):
        """Manifest 文件存在"""
        assert manifest_path.exists(), f"Manifest 文件不存在: {manifest_path}"

    def test_manifest_is_valid_json(self, manifest_path):
        """Manifest 是有效的 JSON"""
        with open(manifest_path, encoding='utf-8') as f:
            data = json.load(f)
        assert data is not None

    @pytest.fixture
    def manifest_data(self, manifest_path):
        with open(manifest_path, encoding='utf-8') as f:
            return json.load(f)

    def test_manifest_has_required_fields(self, manifest_data):
        """Manifest 包含必需字段"""
        required_fields = ["order_id", "customer", "project_name", "created_at", "status", "includes"]
        for field in required_fields:
            assert field in manifest_data, f"Manifest 缺少必需字段: {field}"

    def test_manifest_includes_all_components(self, manifest_data):
        """Manifest 包含所有必需组件"""
        include_types = {item["type"] for item in manifest_data["includes"]}
        required_types = {"recipe", "dataset", "plan", "schema", "inspector", "report"}
        assert include_types == required_types, \
            f"Manifest 组件不完整: 缺少 {required_types - include_types}, 多余 {include_types - required_types}"

    def test_manifest_files_exist(self, manifest_data):
        """Manifest 中引用的文件都存在"""
        fixtures_root_resolved = FIXTURES_ROOT.resolve()
        for item in manifest_data["includes"]:
            file_path = (FIXTURES_ROOT / item["file"]).resolve()
            # 确保路径在 FIXTURES_ROOT 下，防止路径越界
            assert (fixtures_root_resolved in file_path.parents or
                    file_path.parent == fixtures_root_resolved), \
                f"Manifest 引用路径越界: {item['file']}"
            assert file_path.exists(), f"Manifest 引用的文件不存在: {item['file']}"

    def test_manifest_quality_assurance(self, manifest_data):
        """Manifest 包含质检信息"""
        assert "quality_assurance" in manifest_data, "Manifest 缺少 quality_assurance 字段"
        qa = manifest_data["quality_assurance"]
        assert "status" in qa, "quality_assurance 缺少 status 字段"
        assert "inspector_result" in qa, "quality_assurance 缺少 inspector_result 字段"


class TestDataConsistency:
    """验证数据一致性"""

    @pytest.fixture
    def cleaned_df(self):
        return pd.read_csv(FIXTURES_ROOT / "record" / "questionnaire_cleaned.csv")

    def test_age_range(self, cleaned_df):
        """年龄字段在合理范围内"""
        assert "age" in cleaned_df.columns, "缺少 age 字段"
        # 过滤掉缺失值
        valid_ages = cleaned_df["age"][cleaned_df["age"] != -99]
        assert valid_ages.between(18, 70).all(), \
            f"存在超出年龄范围的值: {valid_ages[~valid_ages.between(18, 70)].tolist()}"

    def test_tenure_years_range(self, cleaned_df):
        """工作年限在合理范围内"""
        assert "tenure_years" in cleaned_df.columns, "缺少 tenure_years 字段"
        # 过滤掉缺失值
        valid_tenures = cleaned_df["tenure_years"][cleaned_df["tenure_years"] != -99]
        assert valid_tenures.between(0, 50).all(), \
            f"存在超出工作年限范围的值: {valid_tenures[~valid_tenures.between(0, 50)].tolist()}"

    def test_department_codes(self, cleaned_df):
        """部门编码符合规范"""
        assert "department" in cleaned_df.columns, "缺少 department 字段"
        valid_departments = cleaned_df["department"][
            (cleaned_df["department"] != -99) & (cleaned_df["department"] != -88)
        ]
        valid_codes = {1, 2, 3, 4, 5}
        assert set(valid_departments).issubset(valid_codes), \
            f"存在无效的部门编码: {set(valid_departments) - valid_codes}"

    def test_satisfaction_scale(self, cleaned_df):
        """满意度量表范围正确"""
        assert "satisfaction" in cleaned_df.columns, "缺少 satisfaction 字段"
        valid_satisfaction = cleaned_df["satisfaction"][cleaned_df["satisfaction"] != -99]
        assert valid_satisfaction.between(1, 5).all(), \
            f"存在超出满意度范围的值: {valid_satisfaction[~valid_satisfaction.between(1, 5)].tolist()}"

    def test_binary_fields(self, cleaned_df):
        """二进制字段只包含 0 和 1"""
        binary_fields = ["benefit_insurance", "benefit_vacation", "benefit_medical"]
        for field in binary_fields:
            if field in cleaned_df.columns:
                assert cleaned_df[field].isin([0, 1]).all(), \
                    f"{field} 字段包含非 0/1 的值: {cleaned_df[field][~cleaned_df[field].isin([0, 1])].tolist()}"

    def test_missing_codes_consistent(self, cleaned_df):
        """缺失值编码一致性"""
        missing_codes = [-99, -88]
        for col in cleaned_df.select_dtypes(include='number').columns:
            if col not in ["submit_time"]:
                # 检查数值型字段的缺失值编码
                unique_vals = set(cleaned_df[col].dropna().unique())
                # 确保除预定义缺失码外，其他值不是 -99 或 -88 的变体
                # 实际业务中应确保所有值要么是有效数据，要么是预定义缺失码
                non_missing_values = unique_vals - set(missing_codes)
                # 这里可以添加业务范围验证
                pass


class TestTransformations:
    """验证数据转换规则"""

    @pytest.fixture
    def cleaned_df(self):
        return pd.read_csv(FIXTURES_ROOT / "record" / "questionnaire_cleaned.csv")

    def test_benefits_split(self, cleaned_df):
        """福利多选题拆分正确"""
        # 检查：如果 benefits_raw 包含某个选项，对应的二进制字段应为 1
        for idx, row in cleaned_df.iterrows():
            if pd.notna(row["benefits_raw"]) and row["benefits_raw"] != "":
                if "五险一金" in row["benefits_raw"]:
                    assert row["benefit_insurance"] == 1, \
                        f"行 {idx}: benefits_raw 包含'五险一金'但 benefit_insurance 不为 1"
                if "带薪年假" in row["benefits_raw"]:
                    assert row["benefit_vacation"] == 1, \
                        f"行 {idx}: benefits_raw 包含'带薪年假'但 benefit_vacation 不为 1"
                if "补充医疗" in row["benefits_raw"]:
                    assert row["benefit_medical"] == 1, \
                        f"行 {idx}: benefits_raw 包含'补充医疗'但 benefit_medical 不为 1"

    def test_other_dept_condition(self, cleaned_df):
        """其他部门说明字段逻辑正确"""
        # 只有当 department=5 时，other_dept_specify 才应该有值
        for idx, row in cleaned_df.iterrows():
            if row["department"] != 5:
                assert pd.isna(row["other_dept_specify"]) or row["other_dept_specify"] == "", \
                    f"行 {idx}: department={row['department']} 但 other_dept_specify 有值"


class TestReport:
    """验证报告文件"""

    @pytest.fixture
    def report_path(self):
        return FIXTURES_ROOT / "report" / "questionnaire_cleaning_report.md"

    def test_report_exists(self, report_path):
        """报告文件存在"""
        assert report_path.exists(), f"报告文件不存在: {report_path}"

    def test_report_has_quality_section(self, report_path):
        """报告包含质量检查章节"""
        import re
        content = report_path.read_text(encoding='utf-8')
        assert re.search(r"^##.*数据质量检查", content, re.MULTILINE), \
            "报告缺少数据质量检查章节"

    def test_report_has_recommendations_section(self, report_path):
        """报告包含建议章节"""
        import re
        content = report_path.read_text(encoding='utf-8')
        assert re.search(r"^##\s+\d+\.?\s*建议", content, re.MULTILINE) or \
               re.search(r"^##\s+建议", content, re.MULTILINE), \
            "报告缺少建议章节"
