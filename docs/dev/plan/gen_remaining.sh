#!/bin/bash
PLAN_DIR="/Users/mac/repos/quanttide/domains/quanttide-data/src/qtcloud-data/docs/dev/plan"

# Task 09
cat > "${PLAN_DIR}/09_refactor_test_structure.md" << 'EOF'
# TASK_009: 重构测试结构

## 上下文
- **阶段**: 阶段二 - 测试重构
- **优先级**: P0（必须完成）
- **预计工时**: 2 小时
- **前置任务**: `08_implement_data_schema_model.md`
- **后置任务**: `10_rewrite_workspace_tests.md`

## 实施步骤
1. 创建 `tests/fixtures/business/` 目录
2. 创建 `tests/fixtures/technical/` 目录
3. 按业务概念组织测试文件

## 交付物
- 新测试目录结构
- 设计文档

## 参考
- 原始测试：`tests/test_fixtures.py`
EOF

# Task 10
cat > "${PLAN_DIR}/10_rewrite_workspace_tests.md" << 'EOF'
# TASK_010: 重写 Workspace 测试

## 上下文
- **阶段**: 阶段二 - 测试重构
- **优先级**: P0（必须完成）
- **预计工时**: 1 小时
- **前置任务**: `09_refactor_test_structure.md`
- **后置任务**: `11_rewrite_artifacts_tests.md`

## 任务目标
使用 `quanttide_data_toolkit` 重写 Workspace 测试。

## 实现步骤
1. 创建 `tests/fixtures/business/test_workspace.py`
2. 使用 `Workspace` 领域模型重写测试
3. 确保所有测试通过

## 交付物
- 测试文件：`tests/fixtures/business/test_workspace.py`

## 参考
- 领域模型：`packages/quanttide_data_toolkit/src/quanttide_data_toolkit/workspace.py`
EOF

echo "Generated tasks 09-10"
