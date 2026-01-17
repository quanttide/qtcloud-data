# 业务意图表达清晰度

### 业务意图清晰度

**规则描述**：代码应清晰地表达业务意图，让业务人员能够理解代码的目的和逻辑，避免业务意图被淹没在技术细节中。

**判断标准**：
- 优秀：代码结构与业务概念层次一致，命名直接使用业务术语，业务人员可读
- 警告：业务意图需要结合文档才能理解，存在较多技术细节
- 不合规：业务意图难以理解，代码结构与业务概念严重脱节

**检测方法**：人工审查

**示例**：
```python
# ❌ 业务意图不清晰
# 问题：业务意图是"验证问卷清洗 fixtures 符合规范"
# 但代码变成了 390 行的技术细节检查
class TestFixturesStructure:
    def test_workspace_exists(self):
        assert FIXTURES_ROOT.exists()

    def test_required_subdirectories_exist(self):
        required_dirs = ["plan", "spec", "schema", "processor", "inspector", "record", "report", "manifest"]
        for dir_name in required_dirs:
            dir_path = FIXTURES_ROOT / dir_name
            assert dir_path.exists()

# ✅ 业务意图清晰
# 代码直接表达了"验证工作区结构完整性"的业务意图
class TestWorkspaceStructure:
    """工作区结构完整性验证"""

    def test_workspace_complete(self):
        """工作区包含所有必需目录"""
        structure = WorkspaceStructure(FIXTURES_ROOT)
        assert structure.is_complete(), structure.validation_report()

    def test_business_artifacts_exist(self):
        """业务工件（Plan、Schema、Manifest）存在"""
        artifacts = BusinessArtifacts(FIXTURES_ROOT)
        assert artifacts.has_plan(), "缺少业务意图（Plan）文档"
        assert artifacts.has_schema(), "缺少数据结构（Schema）定义"
        assert artifacts.has_manifest(), "缺少处理清单（Manifest）"

    def test_data_pipeline_complete(self):
        """数据处理流水线完整"""
        pipeline = DataPipeline(FIXTURES_ROOT)
        assert pipeline.has_processor(), "缺少处理器"
        assert pipeline.has_inspector(), "缺少检查器"
        assert pipeline.has_raw_data(), "缺少原始数据"
        assert pipeline.has_cleaned_data(), "缺少清洗后数据"
```

---

### 层次匹配度

**规则描述**：代码结构应与业务概念的层次结构保持一致，避免业务层次被扁平化或过度嵌套。

**判断标准**：
- 优秀：代码层次直接映射业务层次（如：工作区 → 工件 → 数据流）
- 警告：业务层次需要通过注释或命名推断
- 不合规：业务层次完全丢失，代码结构为纯技术层次

**检测方法**：人工审查

**示例**：
```python
# ❌ 层次不匹配
# 业务层次：工作区 → Plan/Schema/Processor → 数据
# 代码层次：TestFixturesStructure → TestPlan/TestSchema/TestInspector/TestData...
class TestFixturesStructure:  # 技术视角："测试结构"
class TestPlan:  # 混乱：Plan 是业务工件，不是测试对象
class TestSchema:  # 同上
class TestDataRecords:  # 同上

# ✅ 层次匹配
# 业务层次：工作区 → 业务工件 → 数据流水线
class TestWorkspace:  # 业务视角：测试工作区
    def test_business_artifacts_complete(self):
        """业务工件完整"""
        pass

class TestBusinessArtifacts:  # 业务工件（Plan、Schema、Manifest）
    def test_plan_clear(self):
        """Plan 清晰表达业务意图"""
        pass

class TestDataPipeline:  # 数据流水线（Processor → Inspector → Data）
    def test_pipeline_executable(self):
        """流水线可执行"""
        pass
```

---

### 术语对齐

**规则描述**：代码中的命名应直接使用业务术语，避免技术术语替代业务概念。

**判断标准**：
- 优秀：类名、方法名、变量名直接使用业务术语
- 警告：部分使用业务术语，部分使用技术术语
- 不合规：主要使用技术术语，业务术语极少

**检测方法**：人工审查

**示例**：
```python
# ❌ 术语不对齐
# 业务概念：Plan（业务意图）、Schema（数据结构）、Inspector（质量检查）
# 代码术语：TestPlan、test_plan_exists、test_schema_is_valid_json
class TestPlan:
    def test_plan_exists(self):  # "exists" 是技术术语
        pass

    def test_schema_is_valid_json(self):  # "valid JSON" 是技术术语
        pass

# ✅ 术语对齐
class TestBusinessIntent:  # 直接使用"业务意图"
    def test_intent_clear(self):  # 使用"清晰"业务术语
        """业务意图清晰表达"""
        pass

class TestDataStructure:  # 使用"数据结构"
    def test_structure_complete(self):  # 使用"完整"
        """数据结构定义完整"""
        pass

class TestQualityCheck:  # 使用"质量检查"
    def test_check_effective(self):  # 使用"有效"
        """质量检查有效"""
        pass
```

---

### 抽象层级合适

**规则描述**：应在合适的抽象层级描述业务逻辑，避免在业务层暴露技术实现细节。

**判断标准**：
- 优秀：使用领域模型或 DSL 描述业务逻辑，技术细节被封装
- 警告：业务逻辑与技术细节混合
- 不合规：业务逻辑被技术细节淹没

**检测方法**：人工审查

**示例**：
```python
# ❌ 抽象层级不合适
# 业务逻辑："验证 Schema 包含必需字段"
# 代码：直接操作 JSON 和文件系统
def test_schema_has_required_fields(self, schema_data):
    required_fields = ["name", "version", "schema", "quality_rules", "transformations"]
    for field in required_fields:
        assert field in schema_data, f"Schema 缺少必需字段: {field}"

# ✅ 抽象层级合适
# 业务逻辑："验证 Schema 完整性"
# 代码：使用领域模型
def test_schema_complete(self):
    """Schema 完整性"""
    schema = DataSchema.from_file(FIXTURES_ROOT / "schema" / "questionnaire_schema.json")
    assert schema.is_complete(), schema.missing_fields_report()

# 领域模型
class DataSchema:
    """数据结构领域模型"""
    REQUIRED_FIELDS = ["name", "version", "schema", "quality_rules", "transformations"]

    def is_complete(self) -> bool:
        """是否完整"""
        return all(field in self.data for field in self.REQUIRED_FIELDS)

    def missing_fields_report(self) -> str:
        """缺失字段报告"""
        missing = self.REQUIRED_FIELDS - set(self.data.keys())
        return f"Schema 缺少必需字段: {missing}"
```

---

### 业务可读性

**规则描述**：代码应能让业务人员理解，不需要深入理解技术实现。

**判断标准**：
- 优秀：业务人员能通过代码理解业务逻辑
- 警告：业务人员需要结合文档理解
- 不合规：业务人员无法理解代码意图

**检测方法**：人工审查（业务人员评审）

**示例**：
```python
# ❌ 业务不可读
# 业务人员看到的是：assert、for 循环、字典操作、条件判断
def test_schema_field_definitions_complete(self, schema_data):
    valid_types = {"string", "integer", "float", "binary", "datetime", "categorical", "text"}
    for field in schema_data["schema"]["fields"]:
        required_attrs = ["name", "type"]
        for attr in required_attrs:
            assert attr in field, f"字段 {field.get('name')} 缺少必需属性: {attr}"
            assert field["type"] in valid_types, f"字段 {field['name']} 的 type 不合法: {field['type']}"

# ✅ 业务可读
# 业务人员看到的是：Schema 字段定义完整性验证
def test_schema_fields_well_defined(self):
    """Schema 字段定义完整"""
    schema = DataSchema.from_file(FIXTURES_ROOT / "schema" / "questionnaire_schema.json")
    assert schema.fields_well_defined(), schema.fields_validation_report()

class DataSchema:
    """数据结构"""

    def fields_well_defined(self) -> bool:
        """字段定义完整"""
        for field in self.fields:
            if not self._field_valid(field):
                return False
        return True

    def fields_validation_report(self) -> str:
        """字段验证报告"""
        issues = []
        for field in self.fields:
            if not self._field_valid(field):
                issues.append(f"字段 {field['name']} 定义不完整")
        return "; ".join(issues)
```

---

## 检查清单

在审查代码的业务意图表达时，请使用以下检查清单：

### 基础要求（必须满足）
- [ ] 代码结构反映了业务概念层次
- [ ] 命名使用业务术语而非技术术语
- [ ] 业务逻辑在合适的抽象层级描述
- [ ] 业务人员能够理解代码意图

### 推荐要求（建议满足）
- [ ] 提供领域模型或 DSL 封装技术细节
- [ ] 测试方法名直接表达业务意图
- [ ] 文档字符串使用业务语言描述

---

## 规则成熟度

审计规则可以按以下阶段逐步完善：

- **阶段 1（基础）**：规则名称 + 描述 + 判断标准
- **阶段 2（可用）**：阶段 1 + 检测方法
- **阶段 3（完善）**：阶段 2 + 示例 + 领域模型示例

---

## 规则层级
**业务规则** - 适用于业务系统代码、测试代码、领域模型代码
