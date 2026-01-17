from pathlib import Path
from typing import Any, Dict
import json


class BusinessArtifacts:
    """业务工件

    业务工件是数据清洗项目中的关键交付物，包括 Plan、Schema、Manifest 等。
    """

    def __init__(self, workspace: 'Workspace'):
        """初始化业务工件

        Args:
            workspace: 工作区对象
        """
        self.workspace = workspace

    def plan_is_clear(self) -> bool:
        """Plan 清晰度检查

        Returns:
            bool: Plan 文件包含数据模型和处理流程章节
        """
        plan_path = self.workspace.get_component("plan")
        plan_files = list(plan_path.glob("*.md"))

        if not plan_files:
            return False

        # 优先选择包含 "plan" 的文件
        plan_file = None
        for f in plan_files:
            if "plan" in f.name.lower():
                plan_file = f
                break
        if not plan_file:
            plan_file = plan_files[0]

        content = plan_file.read_text(encoding='utf-8')
        return "## 数据模型" in content and "## 数据处理流程" in content

    def schema_is_complete(self) -> bool:
        """Schema 完整性检查

        Returns:
            bool: Schema 文件存在且包含必需字段
        """
        schema_path = self.workspace.get_component("schema")
        schema_files = list(schema_path.glob("*.json"))

        if not schema_files:
            return False

        # 选择第一个 JSON 文件
        with open(schema_files[0], encoding='utf-8') as f:
            data = json.load(f)

        required_fields = ["name", "version", "schema", "quality_rules", "transformations"]
        return all(field in data for field in required_fields)

    def schema_is_well_formed(self) -> bool:
        """Schema 格式检查

        Returns:
            bool: Schema 结构完整，包含字段定义
        """
        schema_path = self.workspace.get_component("schema")
        schema_files = list(schema_path.glob("*.json"))

        if not schema_files:
            return False

        # 选择第一个 JSON 文件
        with open(schema_files[0], encoding='utf-8') as f:
            data = json.load(f)

        if "schema" not in data or "fields" not in data["schema"]:
            return False

        fields = data["schema"]["fields"]
        if not isinstance(fields, list) or len(fields) == 0:
            return False

        valid_types = {"string", "integer", "float", "binary", "datetime", "categorical", "text"}
        for field in fields:
            if "name" not in field or "type" not in field:
                return False
            if field["type"] not in valid_types:
                return False

        return True

    def manifest_is_complete(self) -> bool:
        """Manifest 完整性检查

        Returns:
            bool: Manifest 文件存在且包含必需字段
        """
        manifest_path = self.workspace.get_component("manifest")
        manifest_files = list(manifest_path.glob("*.json"))

        if not manifest_files:
            return False

        # 优先选择包含 "cleaning" 的 manifest 文件
        manifest_file = None
        for f in manifest_files:
            if "cleaning" in f.name.lower() and "recipe" not in f.name.lower():
                manifest_file = f
                break
        if not manifest_file:
            manifest_file = manifest_files[0]

        with open(manifest_file, encoding='utf-8') as f:
            data = json.load(f)

        required_fields = ["order_id", "customer", "project_name", "created_at", "status", "includes"]
        return all(field in data for field in required_fields)

    def validation_report(self) -> str:
        """生成验证报告

        Returns:
            str: 详细的验证报告
        """
        issues = []

        if not self.plan_is_clear():
            issues.append("Plan 文件缺少数据模型或数据处理流程章节")

        if not self.schema_is_complete():
            issues.append("Schema 文件不完整，缺少必需字段")

        if not self.schema_is_well_formed():
            issues.append("Schema 文件格式不正确")

        if not self.manifest_is_complete():
            issues.append("Manifest 文件不完整，缺少必需字段")

        if not issues:
            return "✅ 所有业务工件验证通过"

        report = "❌ 业务工件验证失败：\n"
        for issue in issues:
            report += f"  - {issue}\n"
        return report
