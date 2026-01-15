# 量潮数据云 Python 工具箱

基于工程标准的数据清洗 SDK，支持从规范验证到质量检查的完整生命周期管理。

## 安装

```bash
pip install -e .
```

## 快速开始

```python
from pathlib import Path
from qtcloud_data import Workspace, DataCleaningPipeline

# 创建工作空间
workspace = Workspace(
    workspace_name="questionnare_cleanning",
    fixtures_root=Path(__file__).parent / "fixtures"
)

# 验证工作空间
validation_results = workspace.run_full_validation()
for step, (success, message) in validation_results.items():
    print(f"{step}: {'✅' if success else '❌'} {message}")

# 运行流水线
pipeline = DataCleaningPipeline(workspace)
results = pipeline.run_pipeline()

if results["success"]:
    print("流水线执行成功！")
    print(results["steps"]["quality_checks"]["summary"])
else:
    print("流水线执行失败：", results.get("error"))
```

## 核心模块

### Workspace

工作空间管理，负责验证项目结构和规范。

```python
# 初始化工作空间
workspace = Workspace(
    workspace_name="questionnare_cleanning",
    fixtures_root=Path("fixtures")
)

# 单步验证
success, message = workspace.validate_step1_spec_files()
print(message)

# 完整验证
results = workspace.run_full_validation()
```

### DataCleaningPipeline

数据处理流水线，负责执行完整的数据清洗流程。

```python
# 初始化流水线
pipeline = DataCleaningPipeline(workspace)

# 分步执行
raw_data = pipeline.load_raw_data()
pipeline.load_processor()
cleaned_data = pipeline.run_cleaning()
pipeline.save_cleaned_data()

# 验证输出
validation = pipeline.validate_output()
print(validation)

# 质量检查
quality_summary = pipeline.run_quality_checks()
print(quality_summary)

# 一次性运行
results = pipeline.run_pipeline()
```

### QualityChecker

数据质量检查器，验证数据符合规范定义。

```python
from qtcloud_data import QualityChecker
import json

# 加载 schema
with open("schema/questionnare.json", "r") as f:
    schema = json.load(f)

# 创建检查器
checker = QualityChecker(schema)

# 运行检查
results = checker.run_all_checks(data)
summary = checker.get_summary(data)
print(summary)
```

## 测试

运行集成测试：

```bash
cd python_sdk
pytest integrated_tests/test_questionnaire_cleanning.py -v
```

## 工程标准

本 SDK 遵循工程标准的三大核心文档：

- 🏗️ **设计图纸** - 数据模型定义
- ⚙️ **工艺卡** - 数据处理流程
- 🔍 **质检标准** - 数据质量规则

详见：[docs/spec/README.md](../../docs/spec/README.md)
