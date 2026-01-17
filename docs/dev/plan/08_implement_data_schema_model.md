# TASK_008: 实现 DataSchema 数据结构模型

## 上下文

- **阶段**: 阶段一 - 领域模型设计与实现
- **优先级**: P0（必须完成）
- **预计工时**: 1 小时
- **前置任务**: `07_implement_data_pipeline_model.md`
- **后置任务**: `09_refactor_test_structure.md`

## 任务目标

根据设计文档 `04_design_data_schema_model.md` 实现 `DataSchema` 类。

## 实现步骤

### 1. 实现 DataSchema 类

创建文件 `packages/quanttide_data_toolkit/src/quanttide_data_toolkit/data_schema.py`：

```python
import json
import pandas as pd
from pathlib import Path
from typing import Set, Dict


class DataSchema:
    """数据结构定义

    DataSchema 封装了数据的结构定义，包括字段、类型、约束等。

    Attributes:
        schema_path (Path): Schema 文件路径
    """

    VALID_TYPES = {
        "string", "integer", "float", "binary",
        "datetime", "categorical", "text"
    }

    def __init__(self, schema_path: Path):
        """初始化数据结构定义

        Args:
            schema_path: Schema 文件路径
        """
        self.schema_path = Path(schema_path)

    def _load_schema(self) -> Dict:
        """加载 Schema

        Returns:
            Dict: Schema 数据

        Raises:
            FileNotFoundError: 如果 Schema 文件不存在
            json.JSONDecodeError: 如果 Schema 不是有效的 JSON
        """
        with open(self.schema_path, encoding='utf-8') as f:
            return json.load(f)

    def is_complete(self) -> bool:
        """Schema 完整性检查

        检查 Schema 是否包含所有必需字段。

        Returns:
            bool: 如果 Schema 完整，返回 True
        """
        try:
            schema = self._load_schema()
            required_fields = [
                "name", "version", "schema",
                "quality_rules", "transformations"
            ]

            for field in required_fields:
                if field not in schema:
                    return False

            return True
        except (FileNotFoundError, json.JSONDecodeError):
            return False

    def is_well_defined(self) -> bool:
        """Schema 定义检查

        检查字段定义是否完整且类型合法。

        Returns:
            bool: 如果定义完整且合法，返回 True
        """
        try:
            schema = self._load_schema()

            if "fields" not in schema.get("schema", {}):
                return False

            fields = schema["schema"]["fields"]
            if not isinstance(fields, list) or len(fields) == 0:
                return False

            for field in fields:
                if "name" not in field or "type" not in field:
                    return False
                if field["type"] not in self.VALID_TYPES:
                    return False

            return True
        except (FileNotFoundError, json.JSONDecodeError):
            return False

    def fields_match(self, data_columns: Set[str]) -> bool:
        """字段匹配检查

        检查 Schema 定义的字段是否与数据列一致。

        Args:
            data_columns: 数据的列名集合

        Returns:
            bool: 如果字段匹配，返回 True
        """
        try:
            schema = self._load_schema()
            schema_fields = {f["name"] for f in schema["schema"]["fields"]}

            return schema_fields == data_columns
        except (FileNotFoundError, json.JSONDecodeError, KeyError):
            return False

    def get_schema_fields(self) -> Set[str]:
        """获取 Schema 字段

        Returns:
            Set[str]: 字段名称集合
        """
        schema = self._load_schema()
        return {f["name"] for f in schema["schema"]["fields"]}

    def validation_report(self) -> str:
        """生成验证报告

        Returns:
            str: 详细的验证报告，列出所有检查项的结果
        """
        results = {
            "Schema 完整性": self.is_complete(),
            "Schema 定义": self.is_well_defined(),
        }

        report = "数据结构验证报告：\n"
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
from .data_schema import DataSchema

__all__ = ['Workspace', 'BusinessArtifacts', 'DataPipeline', 'DataSchema']
```

### 3. 编写单元测试

创建文件 `packages/quanttide_data_toolkit/tests/quanttide_data_toolkit/test_data_schema.py`：

```python
import pytest
from pathlib import Path
import sys
import pandas as pd

# 添加 src 到路径
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))

from quanttide_data_toolkit import DataSchema


class TestDataSchema:
    """DataSchema 领域模型测试"""

    @pytest.fixture
    def schema_path(self):
        return (
            Path(__file__).parent.parent.parent.parent.parent / "tests" /
            "fixtures" / "workspace" / "schema" / "questionnaire_schema.json"
        )

    @pytest.fixture
    def schema(self, schema_path):
        return DataSchema(schema_path)

    @pytest.fixture
    def cleaned_data(self):
        data_path = (
            Path(__file__).parent.parent.parent.parent.parent / "tests" /
            "fixtures" / "workspace" / "record" / "questionnaire_cleaned.csv"
        )
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

    def test_validation_report(self, schema):
        """验证报告"""
        report = schema.validation_report()
        assert report is not None
        assert isinstance(report, str)
```

## 交付物

- 实现代码：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/data_schema.py`
- 更新包初始化：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/__init__.py`
- 单元测试：`packages/quanttide_data_toolkit/tests/quanttide_data_toolkit/test_data_schema.py`

## 验证标准

- [ ] 代码实现符合设计文档
- [ ] 所有单元测试通过
- [ ] 代码风格符合项目规范

## 参考

- 设计文档：`docs/dev/plan/04_design_data_schema_model.md`
- 原始测试：`tests/test_fixtures.py` 第 98-114 行
