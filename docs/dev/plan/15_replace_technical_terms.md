# TASK_015: 替换技术术语

## 上下文

- **阶段**: 阶段三 - 术语对齐与文档完善
- **优先级**: P1（建议完成）
- **预计工时**: 1 小时
- **前置任务**: `14_create_glossary.md`
- **后置任务**: `16_improve_documentation.md`

## 任务目标

根据术语词典替换代码中的技术术语为业务术语。

## 替换清单

### 测试方法名替换

| 原方法名 | 新方法名 |
|---------|---------|
| test_workspace_exists | test_workspace_complete |
| test_plan_exists | test_plan_clear |
| test_schema_is_valid_json | test_schema_well_formed |
| test_schema_has_required_fields | test_schema_complete |
| test_schema_fields_structure | test_schema_well_defined |
| test_schema_field_definitions_complete | test_schema_well_defined |
| test_inspector_initialization | test_inspector_available |
| test_inspector_validate_schema_compliance | test_inspector_complies_with_schema |
| test_inspector_validate_data_quality | test_data_quality_acceptable |
| test_inspector_validate_business_rules | test_business_rules_complied |

### 文档字符串替换

| 原文档字符串 | 新文档字符串 |
|------------|------------|
| "工作区目录存在" | "工作区包含所有必需组件" |
| "Plan 文件存在" | "Plan 清晰度" |
| "Schema 是有效的 JSON" | "Schema 格式正确" |
| "Schema 包含必需字段" | "Schema 完整性" |
| "Inspector 初始化成功" | "Inspector 可用性" |

## 实施步骤

### 1. 批量替换测试方法名

```bash
# 使用 sed 批量替换
sed -i '' 's/test_workspace_exists/test_workspace_complete/g' tests/fixtures/business/*.py
sed -i '' 's/test_plan_exists/test_plan_clear/g' tests/fixtures/business/*.py
sed -i '' 's/test_schema_is_valid_json/test_schema_well_formed/g' tests/fixtures/business/*.py
```

### 2. 批量替换文档字符串

```bash
sed -i '' 's/工作区目录存在/工作区包含所有必需组件/g' tests/fixtures/business/*.py
sed -i '' 's/Plan 文件存在/Plan 清晰度/g' tests/fixtures/business/*.py
sed -i '' 's/Schema 是有效的 JSON/Schema 格式正确/g' tests/fixtures/business/*.py
```

### 3. 验证替换结果

```bash
# 运行测试确保替换后测试仍然通过
pytest tests/fixtures/business/ -v
```

## 注意事项

1. **只替换业务层测试**：`tests/fixtures/business/`
2. **保留技术层测试**：`tests/fixtures/technical/` 中的技术术语
3. **保持测试通过**：替换后必须运行测试验证

## 验证标准

- [ ] 所有业务层测试方法名使用业务术语
- [ ] 所有业务层测试文档字符串使用业务术语
- [ ] 所有测试仍然通过

## 交付物

- 术语对齐后的代码：`tests/fixtures/business/`

## 参考

- 术语词典：`docs/glossary.md`
