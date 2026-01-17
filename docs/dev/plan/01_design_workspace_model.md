# TASK_001: 设计 Workspace 工作区模型

## 上下文

- **阶段**: 阶段一 - 领域模型设计与实现
- **优先级**: P0（必须完成）
- **预计工时**: 0.5 小时
- **前置任务**: 无
- **后置任务**: `02_design_artifacts_model.md`

## 业务目标

创建 `Workspace` 领域模型，封装工作区完整性验证的业务逻辑。

## 设计要求

### 类定义

```python
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
        pass

    def is_complete(self) -> bool:
        """工作区是否完整

        Returns:
            bool: 如果所有必需组件都存在且为目录，返回 True
        """
        pass

    def validation_report(self) -> str:
        """生成验证报告

        Returns:
            str: 详细的验证报告，列出缺失或不合法的组件
        """
        pass

    def get_component(self, component_name: str) -> Path:
        """获取组件路径

        Args:
            component_name: 组件名称

        Returns:
            Path: 组件的完整路径

        Raises:
            ValueError: 如果组件名称不合法
        """
        pass
```

### 设计要点

1. **业务语义**：
   - 使用 "complete" 而非 "exists" 表达业务意图
   - 使用 "validation_report" 而非 "check" 表达业务意图

2. **封装技术细节**：
   - 文件系统操作（`Path.exists()`, `Path.is_dir()`）封装在方法内部
   - 对外只暴露业务语义接口

3. **错误处理**：
   - 提供友好的错误信息
   - 明确区分缺失组件和不合法组件

## 交付物

- 设计文档：`packages/quanttide_data_toolkit/docs/design/workspace.md`
- 类图：`packages/quanttide_data_toolkit/docs/design/workspace_class_diagram.md`

## 验证标准

- [ ] 类定义符合业务语义
- [ ] 方法命名使用业务术语
- [ ] 封装了所有技术细节
- [ ] 提供清晰的文档字符串

## 参考

- 原始测试：`tests/test_fixtures.py` 第 24-45 行
- QA 审计报告：`docs/qa/report/test_fixtures.md`
