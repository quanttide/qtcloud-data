from pathlib import Path
from typing import Dict, Any
import json


class DataSchema:
    """数据结构定义"""

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
        self._data: Dict[str, Any] = {}

        if self.schema_path.exists():
            with open(self.schema_path, encoding='utf-8') as f:
                self._data = json.load(f)

    def is_complete(self) -> bool:
        """Schema 完整性检查

        Returns:
            bool: Schema 包含所有必需字段
        """
        required_fields = ["name", "version", "schema", "quality_rules", "transformations"]
        return all(field in self._data for field in required_fields)

    def is_well_defined(self) -> bool:
        """Schema 定义检查

        Returns:
            bool: Schema 结构完整且定义正确
        """
        if "schema" not in self._data or "fields" not in self._data["schema"]:
            return False

        fields = self._data["schema"]["fields"]
        if not isinstance(fields, list) or len(fields) == 0:
            return False

        for field in fields:
            if "name" not in field or "type" not in field:
                return False
            if field["type"] not in self.VALID_TYPES:
                return False

        return True

    def fields_match(self, data_columns: set) -> bool:
        """字段匹配检查

        Args:
            data_columns: 数据列名集合

        Returns:
            bool: 数据列名与 Schema 字段匹配
        """
        if not self.is_well_defined():
            return False

        schema_fields = {field["name"] for field in self._data["schema"]["fields"]}
        return schema_fields == data_columns

    def validation_report(self) -> str:
        """生成验证报告

        Returns:
            str: 详细的验证报告
        """
        issues = []

        if not self.is_complete():
            issues.append("Schema 缺少必需字段")

        if not self.is_well_defined():
            issues.append("Schema 结构不完整或定义不正确")

        if not issues:
            return "✅ Schema 验证通过"

        report = "❌ Schema 验证失败：\n"
        for issue in issues:
            report += f"  - {issue}\n"
        return report

    @property
    def data(self) -> Dict[str, Any]:
        """获取 Schema 数据

        Returns:
            Schema 数据字典
        """
        return self._data
