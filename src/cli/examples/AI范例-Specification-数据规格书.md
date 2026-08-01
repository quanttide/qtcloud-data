# AI 范例：数据规格书（Specification）

> 以下是从上一份 DRD（GitHub 用户活动面板）生成的 Specification 范例。
> Specification = Contract（输入输出规格）+ Blueprint（处理流程）

---

# 数据规格书：GitHub 用户活动面板

## Part A：数据契约（Contract）

### A1. 输入契约 — 客户需提供

| 字段名 | 类型 | 含义 | 约束 |
|--------|------|------|------|
| `user_list` | 文件（CSV） | 目标用户 ID 列表 | 必填，单列，每行一个用户 ID |
| `ghtorrent_dump` | 文件（MySQL dump） | GHTorrent 全量数据 | 必填，包含 users/projects/commits/pull_requests/issues/issue_comments 等核心表 |

### A2. 输出契约 — 我们将交付

| 字段名 | 类型 | 含义 | 质量承诺 |
|--------|------|------|---------|
| `user_id_hash` | 文本（32字符） | 去标识化的用户 ID | 脱敏，非空，全表唯一 |
| `login` | 文本 | GitHub 登录名 | 非空 |
| `activity_date` | 日期 | 活动观察日期 | YYYY-MM-DD，非空 |
| `push_count` | 整数 | Push 次数 | ≥0 |
| `pr_count` | 整数 | PR 发起次数 | ≥0 |
| `issue_comment_count` | 整数 | Issue 评论次数 | ≥0 |
| `pr_review_comment_count` | 整数 | PR 评论次数 | ≥0 |
| `pr_merged_count` | 整数 | PR 合并次数 | ≥0 |
| `commit_comment_count` | 整数 | Commit 评论次数 | ≥0 |
| `is_bot` | 整数（0或1） | Bot 标记 | 枚举值 [0, 1]，准确率 ≥ 90% |
| `missing_reason` | 文本 | 无法匹配的原因 | 仅当用户列表中该用户无任何活动记录时填充 |

质量承诺：
- 去重率 100%（同一 user_id_hash + activity_date 不重复）
- 覆盖率 100%（输入用户列表中所有用户均出现）
- Bot 识别准确率 ≥ 90%

---

## Part B：处理蓝图（Blueprint）

### 处理流程

```
步骤 1：数据解压与提取
  ↓
步骤 2：用户匹配与子集筛选
  ↓
步骤 3：多维度日聚合计算
  ↓
步骤 4：Bot 识别
  ↓
步骤 5：面板合并与质量校验
  ↓
步骤 6：脱敏与交付打包
```

### 步骤详解

| 步骤 | 业务逻辑 | 预期产出 |
|------|---------|---------|
| 1. 数据解压与提取 | 解压 MySQL dump，提取 users/projects/commits/pull_requests/issues/issue_comments 六张核心表 | 结构化表数据 |
| 2. 用户匹配与子集筛选 | 用用户 ID 列表匹配 users 表获取内部 user_id，以 user_id 为核心过滤其他五张表 | 用户子集关联数据 |
| 3. 多维度日聚合计算 | 按 user_id + 日期 GROUP BY，计算 push/PR/Issue评论/PR评论/PR合并/Commit评论 六个维度的日计数值 | 六列聚合指标表 |
| 4. Bot 识别 | 基于账户名模式（`[bot]` 后缀）、活动频率、API 类型等特征，用规则+LLM 混合判断 is_bot 标记 | 带 bot 标记的数据 |
| 5. 面板合并与质量校验 | 合并所有维度，验证去重完整性、覆盖率、字段非空约束 | 校验报告 |
| 6. 脱敏与交付打包 | user_id → user_id_hash，按主键排序，输出最终 CSV + 数据处理说明 | 最终交付文件 |
