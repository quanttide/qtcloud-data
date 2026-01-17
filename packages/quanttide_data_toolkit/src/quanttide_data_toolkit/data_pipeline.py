from pathlib import Path
from typing import Any, Dict
import sys
import pandas as pd


class DataPipeline:
    """数据流水线

    数据流水线是数据清洗和验证的完整流程。
    """

    def __init__(self, workspace: 'Workspace'):
        """初始化数据流水线

        Args:
            workspace: 工作区对象
        """
        self.workspace = workspace
        self._inspector = None

    def inspector_available(self) -> bool:
        """Inspector 可用性检查

        Returns:
            bool: Inspector 模块可用
        """
        inspector_path = self.workspace.get_component("inspector")
        inspector_files = list(inspector_path.glob("*inspector.py"))

        if not inspector_files:
            return False

        try:
            inspector_dir = str(inspector_path)
            if inspector_dir not in sys.path:
                sys.path.insert(0, inspector_dir)

            module_name = inspector_files[0].stem
            inspector_module = __import__(module_name)

            if hasattr(inspector_module, 'QuestionnaireInspector'):
                self._inspector = inspector_module.QuestionnaireInspector
                return True
        except Exception:
            return False

        return False

    def inspector_complies_with_schema(self) -> bool:
        """Inspector Schema 合规性检查

        Returns:
            bool: Inspector 验证结果通过
        """
        if not self._inspector:
            if not self.inspector_available():
                return False

        try:
            plan_path = self.workspace.get_component("plan")
            plan_files = list(plan_path.glob("*.md"))
            if not plan_files:
                return False

            inspector = self._inspector(plan_files[0])

            record_path = self.workspace.get_component("record")
            cleaned_files = list(record_path.glob("*cleaned*.csv"))
            if not cleaned_files:
                return False

            data = pd.read_csv(cleaned_files[0])
            result = inspector.validate_schema_compliance(data)

            return result is not None and result.get("status") in ("PASS", "passed")
        except Exception:
            return False

    def data_quality_acceptable(self) -> bool:
        """数据质量可接受性检查

        Returns:
            bool: 数据质量验证通过
        """
        if not self._inspector:
            if not self.inspector_available():
                return False

        try:
            plan_path = self.workspace.get_component("plan")
            plan_files = list(plan_path.glob("*.md"))
            if not plan_files:
                return False

            inspector = self._inspector(plan_files[0])

            record_path = self.workspace.get_component("record")
            cleaned_files = list(record_path.glob("*cleaned*.csv"))
            if not cleaned_files:
                return False

            data = pd.read_csv(cleaned_files[0])
            result = inspector.validate_data_quality(data)

            return result is not None and result.get("status") in ("PASS", "passed")
        except Exception:
            return False

    def business_rules_complied(self) -> bool:
        """业务规则合规性检查

        Returns:
            bool: 业务规则验证通过
        """
        if not self._inspector:
            if not self.inspector_available():
                return False

        try:
            plan_path = self.workspace.get_component("plan")
            plan_files = list(plan_path.glob("*.md"))
            if not plan_files:
                return False

            inspector = self._inspector(plan_files[0])

            record_path = self.workspace.get_component("record")
            cleaned_files = list(record_path.glob("*cleaned*.csv"))
            if not cleaned_files:
                return False

            data = pd.read_csv(cleaned_files[0])
            result = inspector.validate_business_rules(data)

            return result is not None and result.get("status") in ("PASS", "passed")
        except Exception:
            return False

    def validation_report(self) -> str:
        """生成验证报告

        Returns:
            str: 详细的验证报告
        """
        issues = []

        if not self.inspector_available():
            issues.append("Inspector 不可用")

        if not self.inspector_complies_with_schema():
            issues.append("Schema 合规性验证失败")

        if not self.data_quality_acceptable():
            issues.append("数据质量验证失败")

        if not self.business_rules_complied():
            issues.append("业务规则验证失败")

        if not issues:
            return "✅ 数据流水线验证通过"

        report = "❌ 数据流水线验证失败：\n"
        for issue in issues:
            report += f"  - {issue}\n"
        return report
