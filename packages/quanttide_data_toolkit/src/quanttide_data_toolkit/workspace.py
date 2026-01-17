from pathlib import Path
from typing import List


class Workspace:
    """工作区

    工作区是数据清洗项目的完整环境，包含所有必需的组件。

    Attributes:
        root_path (Path): 工作区根目录路径
    """

    REQUIRED_COMPONENTS = [
        "plan",       # 业务意图文件
        "spec",       # 规格说明
        "schema",     # 数据结构定义
        "processor",  # 数据处理器
        "inspector",  # 数据检查器
        "record",     # 数据记录
        "report",     # 质量报告
        "manifest",   # 交付物清单
    ]

    def __init__(self, root_path: Path):
        """初始化工作区

        Args:
            root_path: 工作区根目录路径
        """
        self.root_path = Path(root_path)

    def is_complete(self) -> bool:
        """工作区是否完整

        Returns:
            bool: 如果所有必需组件都存在且为目录，返回 True
        """
        missing_components = self._get_missing_components()
        return len(missing_components) == 0

    def _get_missing_components(self) -> List[str]:
        """获取缺失的组件

        Returns:
            缺失组件名称列表
        """
        missing = []
        for component in self.REQUIRED_COMPONENTS:
            component_path = self.root_path / component
            if not (component_path.exists() and component_path.is_dir()):
                missing.append(component)
        return missing

    def validation_report(self) -> str:
        """生成验证报告

        Returns:
            str: 详细的验证报告，列出缺失或不合法的组件
        """
        missing = self._get_missing_components()

        if len(missing) == 0:
            return "✅ 工作区完整，所有必需组件都存在"

        report = "❌ 工作区不完整，缺失以下组件：\n"
        for component in missing:
            report += f"  - {component}\n"
        return report

    def get_component(self, component_name: str) -> Path:
        """获取组件路径

        Args:
            component_name: 组件名称

        Returns:
            Path: 组件的完整路径

        Raises:
            ValueError: 如果组件名称不合法
        """
        if component_name not in self.REQUIRED_COMPONENTS:
            raise ValueError(f"不合法的组件名称: {component_name}")

        return self.root_path / component_name
