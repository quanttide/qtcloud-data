# TASK_014: 创建业务术语词典

## 上下文

- **阶段**: 阶段三 - 术语对齐与文档完善
- **优先级**: P1（建议完成）
- **预计工时**: 1 小时
- **前置任务**: `13_rewrite_schema_tests.md`
- **后置任务**: `15_replace_technical_terms.md`

## 任务目标

创建业务术语词典，统一使用业务术语。

## 业务术语词典

### 文件：`docs/glossary.md`

```markdown
# 业务术语词典

本文档定义了项目中的业务术语，确保代码和文档使用一致的业务语言。

## 术语对照表

| 技术术语 | 业务术语 | 说明 |
|---------|---------|------|
| exists | complete / available | "exists" 是技术概念，"complete" 表达业务完整性 |
| is_valid_json | well_formed | "valid JSON" 是技术概念，"well formed" 表达业务格式正确性 |
| has_required_fields | complete / well_defined | "required fields" 是技术概念，"complete" 表达业务完整性 |
| has_section | clear | "has section" 是技术概念，"clear" 表达业务清晰度 |
| passes | acceptable | "passes" 是技术概念，"acceptable" 表达业务可接受性 |
| validates | complies_with | "validates" 是技术概念，"complies with" 表达业务合规性 |
| check | validate | "check" 是技术概念，"validate" 表达业务验证 |

## 类命名

| 技术命名 | 业务命名 | 说明 |
|---------|---------|------|
| TestFixturesStructure | TestWorkspace | 工作区完整性验证 |
| TestDataRecords | TestDataSchema | 数据结构验证 |
| TestInspector | TestDataPipeline | 数据流水线验证 |

## 方法命名

### Workspace 相关

| 技术命名 | 业务命名 |
|---------|---------|
| test_workspace_exists | test_workspace_complete |
| test_required_subdirectories_exist | test_workspace_complete |

### Artifacts 相关

| 技术命名 | 业务命名 |
|---------|---------|
| test_plan_exists | test_plan_clear |
| test_schema_is_valid_json | test_schema_well_formed |
| test_schema_has_required_fields | test_schema_complete |
| test_manifest_files_exist | test_manifest_complete |

### DataPipeline 相关

| 技术命名 | 业务命名 |
|---------|---------|
| test_inspector_initialization | test_inspector_available |
| test_inspector_validate_schema_compliance | test_inspector_complies_with_schema |
| test_inspector_validate_data_quality | test_data_quality_acceptable |

## 使用指南

1. 优先使用业务术语
2. 只有在表达技术实现细节时才使用技术术语
3. 保持代码、文档、测试的术语一致性
```

## 验证标准

- [ ] 术语词典包含所有关键术语
- [ ] 术语对照清晰明确
- [ ] 提供使用指南

## 交付物

- 术语词典：`docs/glossary.md`

## 参考

- QA 审计报告：`docs/qa/report/test_fixtures.md`
