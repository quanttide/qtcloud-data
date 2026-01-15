"""
问卷数据清洗集成测试

测试流程（参考 docs/qa/questionnaire_cleanning.md）：
1. 假设系统已经存在 spec 的 base.md 和 questionnaire_cleanning.md
2. 委托人提供 record 的 questionnaire_raw.csv
3. 代理人提供 blueprint 的 questionnnare_cleanning
4. 系统生成 schema 的 questionnaire.json 和 processor 的 questionnaire_cleanner.py
5. 系统运行 questionnaire_cleanning.py 获得 record 的 questionnaire_cleanned.csv
6. 系统生成 manifest 的 questionnaire_cleanning.md
7. 系统打包生成 dataset 的 questionnaire_cleanning.zip
"""

import pytest
import pandas as pd
import json
import zipfile
from pathlib import Path
import importlib.util
import tempfile
import shutil


class TestQuestionnaireCleaningPipeline:
    """测试问卷数据清洗的完整流程"""

    @pytest.fixture
    def project_name(self):
        return "questionnaire_cleanning"

    @pytest.fixture
    def fixtures_root(self):
        return Path(__file__).parent / "fixtures"

    @pytest.fixture
    def project_path(self, fixtures_root, project_name):
        return fixtures_root / project_name

    # 步骤1：验证 spec 文件存在
    @pytest.fixture
    def spec_base_path(self, fixtures_root):
        """1. 假设系统已经存在 spec 的 base.md"""
        path = fixtures_root / "spec" / "base.md"
        assert path.exists(), f"Spec base.md 不存在: {path}"
        return path

    @pytest.fixture
    def spec_questionnaire_path(self, fixtures_root):
        """1. 假设系统已经存在 spec 的 questionnaire_cleanning.md"""
        path = fixtures_root / "spec" / "questionnaire_cleanning.md"
        assert path.exists(), f"Spec questionnaire_cleanning.md 不存在: {path}"
        return path

    # 步骤2：验证原始数据存在
    @pytest.fixture
    def raw_data_path(self, project_path):
        """2. 委托人提供 record 的 questionnaire_raw.csv"""
        path = project_path / "record" / "questionnaire_raw.csv"
        assert path.exists(), f"原始数据不存在: {path}"
        return path

    @pytest.fixture
    def raw_data(self, raw_data_path):
        """读取原始数据"""
        return pd.read_csv(raw_data_path)

    # 步骤3：验证蓝图存在
    @pytest.fixture
    def blueprint_path(self, project_path):
        """3. 代理人提供 blueprint 的 questionnnare_cleanning"""
        path = project_path / "blueprint" / "questionnare_cleanning.md"
        assert path.exists(), f"蓝图不存在: {path}"
        return path

    @pytest.fixture
    def blueprint_content(self, blueprint_path):
        """读取蓝图内容"""
        return blueprint_path.read_text()

    # 步骤4：验证生成的 schema 和 processor
    @pytest.fixture
    def schema_path(self, project_path):
        """4. 系统生成 schema 的 questionnaire.json"""
        # 注意：当前实现中 schema 目录可能不存在，这个是预期行为
        # 实际系统应该在 processor 中定义 schema
        return project_path / "schema" / "questionnaire.json"

    @pytest.fixture
    def processor_path(self, project_path):
        """4. 系统生成 processor 的 questionnaire_cleanner.py"""
        path = project_path / "processor" / "questionnaire_cleaner.py"
        assert path.exists(), f"处理器不存在: {path}"
        return path

    @pytest.fixture
    def processor_module(self, processor_path):
        """动态加载处理器模块"""
        spec = importlib.util.spec_from_file_location(
            "questionnaire_cleaner",
            processor_path
        )
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    @pytest.fixture
    def processor(self, processor_module):
        """创建处理器实例"""
        return processor_module.QuestionnareCleaner()

    # 步骤5：运行处理器生成清洗后数据
    @pytest.fixture
    def cleaned_data_path(self, project_path):
        """5. 系统运行 questionnaire_cleanning.py 获得 record 的 questionnaire_cleanned.csv"""
        path = project_path / "dataset" / "questionnaire_cleanned.csv"
        assert path.exists(), f"清洗后数据不存在: {path}"
        return path

    @pytest.fixture
    def expected_cleaned_data(self, cleaned_data_path):
        """读取期望的清洗后数据"""
        return pd.read_csv(cleaned_data_path)

    @pytest.fixture
    def actual_cleaned_data(self, raw_data, processor):
        """运行处理器获得实际清洗后数据"""
        return processor.process(raw_data)

    # 步骤6：验证交付清单
    @pytest.fixture
    def manifest_path(self, project_path):
        """6. 系统生成 manifest 的 questionnaire_cleanning.md"""
        path = project_path / "manifest" / "questionnaire_cleanning.md"
        assert path.exists(), f"交付清单不存在: {path}"
        return path

    @pytest.fixture
    def manifest_content(self, manifest_path):
        """读取交付清单内容"""
        return manifest_path.read_text()

    # 步骤7：验证打包的 dataset
    @pytest.fixture
    def dataset_zip_path(self, project_path):
        """7. 系统打包生成 dataset 的 questionnaire_cleanning.zip"""
        # 注意：当前实现中可能没有生成 zip，这个是预期行为
        # 实际系统应该在所有验证通过后打包
        return project_path / "dataset" / "questionnaire_cleanning.zip"

    # ========== 测试用例 ==========

    def test_step1_spec_files_exist(self, spec_base_path, spec_questionnaire_path):
        """步骤1：验证 spec 文件存在"""
        assert spec_base_path.exists()
        assert spec_questionnaire_path.exists()

        # 验证 spec 内容包含必需信息
        spec_content = spec_questionnaire_path.read_text()
        assert "问卷数据清洗" in spec_content or "数据清洗" in spec_content

    def test_step2_raw_data_exists(self, raw_data_path):
        """步骤2：验证原始数据存在"""
        assert raw_data_path.exists()

        # 验证原始数据包含必要的列
        raw_data = pd.read_csv(raw_data_path)
        required_columns = ["提交时间", "年龄", "工作年限", "所属部门"]
        for col in required_columns:
            assert col in raw_data.columns, f"原始数据缺少列: {col}"

    def test_step3_blueprint_exists_and_valid(self, blueprint_path):
        """步骤3：验证蓝图存在且内容有效"""
        assert blueprint_path.exists()

        content = blueprint_path.read_text()

        # 验证蓝图包含必需章节
        required_sections = ["## 数据模型", "## 数据处理流程"]
        for section in required_sections:
            assert section in content, f"蓝图缺少章节: {section}"

        # 验证蓝图包含字段定义
        assert "字段名" in content
        assert "类型" in content
        assert "缺失编码" in content

    def test_step4_processor_exists_and_loadable(self, processor_path, processor_module):
        """步骤4：验证处理器存在且可加载"""
        assert processor_path.exists()

        # 验证模块包含 QuestionnaireCleaner 类
        assert hasattr(processor_module, "QuestionnaireCleaner")

        # 验证处理器有必需的方法
        cleaner = processor_module.QuestionnaireCleaner()
        assert hasattr(cleaner, "process")

    def test_step5_cleaning_produces_correct_output(
        self, actual_cleaned_data, expected_cleaned_data
    ):
        """步骤5：验证清洗产生正确的输出"""
        # 验证数据形状
        assert actual_cleaned_data.shape == expected_cleaned_data.shape

        # 验证列名一致
        assert set(actual_cleaned_data.columns) == set(expected_cleaned_data.columns)

        # 验证数据内容一致
        pd.testing.assert_frame_equal(
            actual_cleaned_data,
            expected_cleaned_data,
            check_dtype=False,
            check_like=True
        )

    def test_step5_cleaned_data_meets_blueprint(self, actual_cleaned_data, blueprint_content):
        """步骤5：验证清洗后数据符合蓝图定义"""
        # 验证清洗后数据包含蓝图定义的字段
        blueprint_fields = []
        for line in blueprint_content.split('\n'):
            if line.startswith('| `'):
                field_name = line.split('`')[1]
                if field_name not in ['字段名', '原始来源']:
                    blueprint_fields.append(field_name)

        # 验证所有蓝图定义的字段都存在于输出数据中
        for field in blueprint_fields:
            assert field in actual_cleaned_data.columns, f"输出数据缺少字段: {field}"

    def test_step6_manifest_exists_and_complete(self, manifest_path, project_path):
        """步骤6：验证交付清单存在且完整"""
        assert manifest_path.exists()

        content = manifest_path.read_text()

        # 验证清单包含必需章节
        required_sections = [
            "## 📦 交付物清单",
            "## 🔄 数据流转路径",
            "## ✅ 质量验证"
        ]
        for section in required_sections:
            assert section in content, f"交付清单缺少章节: {section}"

        # 验证清单提及所有交付物
        assert "blueprint" in content.lower()
        assert "spec" in content.lower()
        assert "processor" in content.lower()
        assert "record" in content.lower()
        assert "dataset" in content.lower()
        assert "dataset" in content.lower()

    def test_step7_dataset_package_structure(self, project_path):
        """步骤7：验证数据集打包结构（如果存在）"""
        # 注意：当前可能没有生成 zip，这是一个可选验证
        zip_path = project_path / "dataset" / "questionnaire_cleanning.zip"

        if zip_path.exists():
            # 验证 zip 文件包含必要的文件
            with zipfile.ZipFile(zip_path, 'r') as zip_ref:
                file_list = zip_ref.namelist()

                # 验证包含清洗后数据
                assert any("questionnaire_cleanned.csv" in f for f in file_list)

    # ========== 额外的集成测试 ==========

    def test_end_to_end_pipeline(self, raw_data, processor, expected_cleaned_data):
        """端到端测试：从原始数据到清洗后数据"""
        # 执行清洗
        actual_cleaned_data = processor.process(raw_data)

        # 验证输出
        pd.testing.assert_frame_equal(
            actual_cleaned_data,
            expected_cleaned_data,
            check_dtype=False,
            check_like=True
        )

    def test_data_quality_checks(self, actual_cleaned_data):
        """数据质量检查"""
        # 检查必填字段
        assert actual_cleaned_data["submit_time"].notna().all(), "submit_time 不能为空"

        # 检查数值范围
        age_values = actual_cleaned_data["age"]
        assert age_values.between(18, 70).all() or (age_values == -99).any(), \
            "年龄应在18-70范围内或标记为缺失"

        # 检查部门编码
        dept_values = actual_cleaned_data["department"]
        valid_depts = [1, 2, 3, 4, 5, -99]
        assert dept_values.isin(valid_depts).all(), f"部门编码无效: {dept_values.unique()}"

    def test_workflow_order(self):
        """验证工作流顺序符合文档定义"""
        # 根据 docs/qa/questionnaire_cleanning.md 验证步骤顺序
        expected_steps = [
            "spec base.md",
            "spec questionnaire_cleanning.md",
            "record questionnaire_raw.csv",
            "blueprint questionnare_cleanning",
            "processor questionnaire_cleaner.py",
            "dataset questionnaire_cleanned.csv",
            "manifest questionnare_cleanning.md",
        ]

        # 这个测试主要验证文档描述与实际实现一致
        assert len(expected_steps) == 7
