# TASK_013: 重写 DataSchema 和 Report 测试

## 上下文

- **阶段**: 阶段二 - 测试重构
- **优先级**: P0（必须完成）
- **预计工时**: 1 小时
- **前置任务**: `12_rewrite_data_pipeline_tests.md`
- **后置任务**: `14_create_glossary.md`

## 任务目标

使用 `quanttide_data_toolkit` 重写 DataSchema 和 Report 测试。

## 实施步骤

### 1. 创建 DataSchema 测试文件

创建文件 `tests/fixtures/business/test_data_schema.py`：

```python
"""数据结构测试

本模块测试数据结构的完整性和一致性，使用 quanttide_data_toolkit。
"""

import pytest
from pathlib import Path
import pandas as pd
from quanttide_data_toolkit import DataSchema

FIXTURES_ROOT = Path(__file__).parent.parent / "workspace"


class TestDataSchema:
    """数据结构验证

    DataSchema 封装了数据的结构定义。
    """

    @pytest.fixture
    def schema_path(self):
        """Schema 文件路径 fixture"""
        return FIXTURES_ROOT / "schema" / "questionnaire_schema.json"

    @pytest.fixture
    def schema(self, schema_path):
        """DataSchema fixture"""
        return DataSchema(schema_path)

    @pytest.fixture
    def cleaned_data(self):
        """清洗后数据 fixture"""
        data_path = FIXTURES_ROOT / "record" / "questionnaire_cleaned.csv"
        return pd.read_csv(data_path)

    def test_schema_complete(self, schema):
        """Schema 完整性"""
        assert schema.is_complete()

    def test_schema_well_defined(self, schema):
        """Schema 定义"""
        assert schema.is_well_defined()

    def test_fields_match(self, schema, cleaned_data):
        """字段匹配"""
        data_columns = set(cleaned_data.columns)
        assert schema.fields_match(data_columns)

    def test_get_schema_fields(self, schema):
        """获取 Schema 字段"""
        fields = schema.get_schema_fields()
        assert isinstance(fields, set)
        assert len(fields) > 0
```

### 2. 创建 Report 测试文件

创建文件 `tests/fixtures/business/test_report.py`：

```python
"""质量报告测试

本模块测试质量报告的完整性和准确性。
"""

import pytest
from pathlib import Path
import re

FIXTURES_ROOT = Path(__file__).parent.parent / "workspace"


class TestReport:
    """质量报告验证

    质量报告是数据清洗项目的交付物。
    """

    @pytest.fixture
    def report_path(self):
        """报告文件路径 fixture"""
        return FIXTURES_ROOT / "report" / "questionnaire_cleaning_report.md"

    def test_report_complete(self, report_path):
        """报告完整性"""
        content = report_path.read_text(encoding='utf-8')

        required_sections = [
            "## 概述",
            "## 1. 数据概览",
            "## 2. 数据转换说明",
            "## 3. 数据统计",
            "## 5. 异常记录说明",
            "## 6. 数据交付物清单"
        ]

        for section in required_sections:
            assert section in content, f"报告缺少章节: {section}"

    def test_report_has_quality_check(self, report_path):
        """报告包含数据质量检查章节"""
        content = report_path.read_text(encoding='utf-8')
        assert re.search(r"^##.*数据质量检查", content, re.MULTILINE), \
            "报告缺少数据质量检查章节"

    def test_report_quality_check_passed(self, report_path):
        """报告中的质量检查结果为通过"""
        content = report_path.read_text(encoding='utf-8')
        assert re.search(r"质量检查结果.*✅.*通过", content, re.DOTALL), \
            "报告中的质量检查结果应为通过"
```

## 交付物

- 测试文件：`tests/fixtures/business/test_data_schema.py`
- 测试文件：`tests/fixtures/business/test_report.py`

## 参考

- 领域模型：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/data_schema.py`
- 原始测试：`tests/test_fixtures.py` 第 116-163 行、365-458 行
