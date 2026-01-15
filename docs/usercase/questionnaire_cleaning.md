# 问卷数据清洗工作手册

## 文档信息

| 项目 | 内容 |
|------|------|
| 版本 | v1.0 |
| 适用场景 | 问卷数据清洗 |
| 依赖工具 | Python, Pandas, AI 辅助工具 |
| 核心目标 | 在人类主导下，借助 AI 提升清洗方案设计与代码实现效率，确保交付物可靠、可审计、可复现 |

## 目录

- [角色定义](#角色定义)
- [工作流程概览](#工作流程概览)
- [详细流程](#详细流程)
- [反馈与迭代](#反馈与迭代)

---

## 角色定义

| 角色 | 职责 |
|------|------|
| **委托人（业务方）** | 提供原始数据 + 业务需求，验收最终结果 |
| **数据工程师（执行者）** | 设计清洗逻辑、编写/审核代码、验证输出、打包交付 |
| **AI 助手（工具）** | 根据结构化输入生成草稿（蓝图、代码、文档），不直接执行关键操作 |

---

## 工作流程概览

```
阶段1: 需求对齐与原始数据探查
    ↓
阶段2: 撰写清洗蓝图（Blueprint）
    ↓
阶段3: AI 生成处理器代码草稿
    ↓
阶段4: 本地运行与验证
    ↓
阶段5: 生成交付清单（Manifest）
    ↓
阶段6: 打包与移交
    ↓
反馈与迭代（闭环）
```

---

## 详细流程

### 阶段 1️⃣：需求对齐与原始数据探查

**目标**: 明确"洗什么"和"怎么算干净"

**输入**:
- `record/questionnaire_raw.csv`（原始数据文件）
- 业务说明（可为自然语言）

**输出**:
- `spec/questionnaire_cleaning_draft.md`（初步需求清单）

#### 参与方动作

##### 委托人
- ✅ 提供原始文件：`record/questionnaire_raw.csv`
- ✅ 提供业务说明，例如："年龄需为整数，部门需标准化为预设列表"

##### 数据工程师
- 🔍 用 Pandas / Polars 快速探查数据：
  - 列名、缺失率、唯一值、异常分布
- 🔍 识别潜在问题：
  - 编码错误（GBK/UTF-8）
  - 多选题合并
  - 自由文本混入等

##### AI 助手
- 🤖 输入原始 CSV 片段 → AI 建议字段类型与可能清洗规则
- 🤖 将委托人口语需求转为结构化 checklist，例如：
  - ✅ 年龄：整数
  - ✅ 部门：枚举值

**验证检查点**:
- [ ] 原始数据可正常读取
- [ ] 业务需求已明确记录
- [ ] 数据质量问题已初步识别

---

### 阶段 2️⃣：撰写清洗蓝图（Blueprint）

**目标**: 将需求转化为机器可理解的处理说明书

**输入**:
- 需求清单（阶段1输出）
- 原始数据样本

**输出**:
- `blueprint/questionnaire_cleaning.md`（人类审核后定稿）

#### 参与方动作

##### 数据工程师
- ✍️ 在模板中填写结构化表格，例如：

  ```markdown
  ## 数据模型

  | 字段名 | 类型 | 原始来源 | 清洗规则 |
  |--------|------|----------|----------|
  | age | int | 年龄 | 1. 移除非数字字符 2. 转整数 3. 若 >100，设为 null |
  | dept | str | 所属部门 | 1. 去除前后空格 2. 模糊匹配到标准部门列表 3. 无法匹配则标记为 "未知" |
  ```

##### AI 助手
- 🤖 输入需求清单 → AI 生成蓝图草稿
- 🤖 输入原始数据样本 → AI 建议"缺失编码"策略（如空字符串、"N/A"、-1）
- 🤖 自动检查是否覆盖所有 required columns（来自配置）

**验证检查点**:
- [ ] 蓝图包含必需章节：数据模型、数据处理流程
- [ ] 每个字段的清洗规则清晰明确
- [ ] 所有必需列都已定义
- [ ] 缺失值处理策略已确定

---

### 阶段 3️⃣：AI 生成处理器代码草稿

**目标**: 快速获得可运行的代码基础，减少样板代码编写

**输入**:
- `blueprint/questionnaire_cleaning.md`
- `record/questionnaire_raw.csv`（样本）

**输出**:
- `processor/questionnaire_cleaner.py`（含 `QuestionnaireCleaner.process()` 方法）
- `schema/questionnaire.json`（字段名、类型、是否 nullable）

#### 参与方动作

##### AI 助手（由工程师触发）
- 🤖 输入：blueprint.md + raw_data_sample.csv
- 🤖 输出：
  - `processor/questionnaire_cleaner.py`
  - `schema/questionnaire.json`

##### 数据工程师
- 🛠️ 审查 AI 生成的代码：
  - 是否正确处理边界情况？（如空值、极端值）
  - 是否有性能隐患？（如逐行 apply vs 向量化）
  - 是否缺少日志或异常捕获？
- 🛠️ 修改并完善代码，确保逻辑严谨

> ⚠️ **关键原则**: AI 生成 ≠ 可直接使用，必须人工审查。

**验证检查点**:
- [ ] 代码可正常运行，无语法错误
- [ ] 边界情况处理正确
- [ ] 性能满足要求
- [ ] 包含必要的异常处理和日志

---

### 阶段 4️⃣：本地运行与验证

**目标**: 确保清洗结果符合预期

**输入**:
- `processor/questionnaire_cleaner.py`
- `record/questionnaire_raw.csv`

**输出**:
- `dataset/questionnaire_cleaned.csv`

#### 参与方动作

##### 数据工程师
- ▶️ 在 Jupyter Notebook 或 IDE 中加载 `questionnaire_cleaner.py`
- ▶️ 运行 `cleaner.process(raw_df)`，观察中间结果
- ▶️ 对比清洗前后关键字段（如年龄分布、部门覆盖率）
- ▶️ 使用断言或单元测试验证规则，例如：
  ```python
  assert cleaned_df['age'].min() >= 18 or pd.isna(cleaned_df['age'].min())
  ```

##### AI 助手（可选）
- 🤖 输入清洗前后样本 → AI 自动生成验证建议
  - 例如："发现 5 行年龄为负，是否异常？"

**验证检查点**:
- [ ] 清洗后数据保存为 `dataset/questionnaire_cleaned.csv`
- [ ] 数据质量符合业务要求
- [ ] 边界情况测试通过
- [ ] 单元测试覆盖关键规则

---

### 阶段 5️⃣：生成交付清单（Manifest）

**目标**: 提供完整、透明的交付说明

**输入**:
- 所有生成产物（原始数据、蓝图、处理器、清洗后数据、schema）

**输出**:
- `manifest/questionnaire_cleaning.md`

#### 参与方动作

##### AI 助手
- 🤖 自动汇总：
  - 使用了哪些输入（raw data 路径）
  - 应用了哪些规则（引用 blueprint 章节）
  - 产出物列表（cleaned CSV, schema, processor）
  - 质量指标（总行数、缺失率、清洗失败率）
- 🤖 生成初稿：`manifest/questionnaire_cleaning.md`

##### 数据工程师
- ✏️ 补充关键说明：
  - "部门标准化依赖外部映射表 v2.1"
  - "因原始'工作年限'含文字描述，仅保留数字部分"
- ✏️ 确保清单包含所有必需章节：
  - 📦 交付物清单
  - 🔄 流转路径
  - ✅ 质量验证

**验证检查点**:
- [ ] 交付清单包含所有必需章节
- [ ] 关键说明已补充完整
- [ ] 质量指标已计算和记录

---

### 阶段 6️⃣：打包与移交

**目标**: 形成自包含、可审计的交付包

**输入**:
- 所有交付物文件

**输出**:
- `dataset/questionnaire_cleaning.zip`

#### 参与方动作

##### 数据工程师
- 📦 创建 ZIP 包，包含：
  ```
  questionnaire_cleaning.zip
  ├── record/
  │   └── questionnaire_raw.csv
  ├── blueprint/
  │   └── questionnaire_cleaning.md
  ├── processor/
  │   └── questionnaire_cleaner.py
  ├── schema/
  │   └── questionnaire.json
  ├── dataset/
  │   └── questionnaire_cleaned.csv
  └── manifest/
      └── questionnaire_cleaning.md
  ```
- 📦 （可选）添加 `README.txt` 说明版本、生成时间、负责人
- 📦 计算 SHA256 校验和，确保完整性

- 📤 移交委托人，附带简要解读：
  - "清洗后共 1,200 行，部门字段标准化完成，年龄异常已处理。"

**验证检查点**:
- [ ] ZIP 包结构完整
- [ ] SHA256 校验和已计算
- [ ] 交付说明清晰

---

## 反馈与迭代（闭环）

若委托人反馈"某字段处理不对"：

1. **工程师生回溯**
   - 是 spec 不清？
   - 蓝图遗漏？
   - 还是代码 bug？

2. **更新对应环节**
   - 更新 spec / 蓝图 / 代码
   - 重新走流程验证

3. **积累案例**
   - 形成"常见问题 → spec 写法"知识库
   - 提升未来 AI 生成质量

---

## 附录

### 相关文档
- `docs/handbook/README.md` - 数据清洗工作手册
- `docs/qa/questionnaire_cleaning.md` - 测试流程说明
- `docs/spec/README.md` - 工程标准文档

### 示例项目
完整的示例项目位于 `tests/fixtures/questionnaire_cleaning/`，包含：
- 完整的测试数据
- 参考实现
- 集成测试用例
