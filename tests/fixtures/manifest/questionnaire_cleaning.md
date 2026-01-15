# 问卷数据清洗交付清单

## 📦 交付物清单

### 1. 蓝图（Blueprint）
📄 `blueprint/questionnare_cleanning.md`

**内容说明**：设计图纸，定义清洗后数据集的逻辑结构与语义规则
**关键要素**：
- 11个标准化字段的完整定义
- 部门编码映射表（1=生产, 2=研发, 3=销售, 4=职能, 5=其他）
- 福利多选题拆分规则（五险一金、带薪年假、补充医疗）
- 缺失值编码体系（-99=未回答, -88=不适用）
- 逻辑约束与业务规则

**用途**：作为清洗工作的唯一权威依据，业务与技术团队的共识协议

### 2. 规范（Spec）
📄 `spec/questionnaire_cleanning.md`

**内容说明**：清洗流程标准，定义从需求对齐到交付的完整方法论
**关键要素**：
- 5个标准化阶段（需求对齐 → 元数据标准化 → 结构化清洗 → 质量验证 → 交付）
- 角色分工建议（项目经理、数据架构师、数据工程师、质量保证）
- 常见陷阱提醒（缺失值混淆、反向计分题、逻辑矛盾等）

**用途**：指导清洗工作的执行，确保流程一致性

### 3. 处理器（Processor）
🐍 `processor/questionnare_cleaner.py`

**内容说明**：可执行清洗代码，实现从原始数据到标准化数据的完整转换
**关键要素**：
- `QuestionnareCleaner` 类，封装完整清洗逻辑
- 7个私有方法处理不同清洗阶段
- 部门映射表、福利选项列表等配置化管理
- 支持反向计分、多选题拆分、文本提取等复杂逻辑

**用途**：可复现的清洗流程，直接输入原始数据获得清洗结果


### 4. 原始数据（Raw Record）
📊 `record/questionnare_raw.csv`

**内容说明**：未清洗的原始问卷数据，模拟真实采集场景
**关键要素**：
- 10条测试记录，覆盖主要数据模式
- 包含典型问题：单位混杂（"35岁"、"8年"）、时间格式不统一、多选题逗号分隔、"其他"选项含〗括号标注
- 缺失值与异常值样本（年龄17、满意度缺失等）

**用途**：测试清洗逻辑正确性，验证各种边界情况


### 5. 清洗后数据（Cleaned Dataset）
📊 `dataset/questionnare_cleanned.csv`

**内容说明**：标准化清洗后的最终数据集，符合蓝图定义
**关键要素**：
- 11个标准化字段（submit_time, age, tenure_years, department, satisfaction, workload, benefits_raw, benefit_insurance, benefit_vacation, benefit_medical, other_dept_specify）
- 统一格式：时间 `YYYY-MM-DD HH:MM:SS`、数值纯数字、部门编码化
- 缺失值编码：-99（未回答）
- 反向计分已转换：workload = 6 - 原值

**用途**：交付给业务方的最终数据产品


## 🔄 数据流转路径

```
原始数据 (record/questionnare_raw.csv)
    ↓
处理器 (processor/questionnare_cleaner.py)
    ↓
清洗后数据 (dataset/questionnare_cleanned.csv)
    ↑
蓝图 (blueprint/questionnare_cleanning.md)
    ↑
规范 (spec/questionnaire_cleanning.md)
```


## ✅ 质量验证

### 完整性检查
- [x] 字段数：原始7列 → 清洗后11列
- [x] 样本量：10条记录完整保留
- [x] 无数据丢失或重复

### 格式一致性
- [x] 时间格式：统一为 `YYYY-MM-DD HH:MM:SS`
- [x] 数值字段：剥离单位、符号
- [x] 部门编码：文本映射为1-5
- [x] 缺失值：NaN替换为-99

### 逻辑验证
- [x] 年龄范围：17-60（含1条异常保留）
- [x] 工作年限：0.5-35（均在合理范围）
- [x] 满意度/工作负荷：1-5（workload已反向计分）
- [x] 部门与说明一致性：仅department=5时有other_dept_specify


## 📋 使用说明

### 运行清洗流程
```python
import pandas as pd
from processor.questionnare_cleaner import QuestionnareCleaner

# 读取原始数据
raw_df = pd.read_csv('record/questionnare_raw.csv')

# 执行清洗
cleaner = QuestionnareCleaner()
cleaned_df = cleaner.process(raw_df)

# 保存清洗后数据
cleaned_df.to_csv('dataset/questionnare_cleanned.csv', index=False)
```

### 自定义配置
修改 `processor/questionnare_cleaner.py` 中的配置项：
```python
DEPT_MAPPING = {...}  # 部门映射
BENEFIT_OPTIONS = [...]  # 福利选项列表
```


## 📝 备注

- 本交付物已通过完整测试，可直接用于生产环境
- 清洗代码包含详细注释，便于后续维护
- 所有文件统一使用UTF-8编码
- 数据模型定义严格遵循工程标准（设计图纸 + 工艺卡 + 质检标准）
