## 输入契约

| 字段名 | 类型 | 业务含义 | 约束条件 |
|--------|------|----------|----------|
| report_id | string | 报告唯一标识 | 必填，不可为空 |
| institution | string | 检测机构名称 | 必填，枚举（SGS/华测/谱尼等） |
| sample_id | string | 样品编号 | 必填，不可为空 |
| sample_name | string | 样品名称 | 可选 |
| original_indicator_name | string | 原始指标名称 | 必填，不可为空 |
| time_point | string | 检测时间点 | 必填，格式如T0/T1/T2 |
| group_name | string | 组别名称 | 必填，如空白组/实验组 |
| mean_value | double | 均值 | 可选，保留原始小数位数 |
| std_dev | double | 标准差 | 可选 |
| change_rate | double | 变化率（百分比） | 可选，保留原始精度如30.37 |
| p_value | double | P值 | 可选 |
| unit | string | 单位 | 可选 |
| page_number | int | 来源页码 | 必填，正整数 |
| location_description | string | 位置描述（如表格坐标） | 可选 |

## 输出契约

| 字段名 | 类型 | 业务含义 | 质量承诺 |
|--------|------|----------|----------|
| standard_report_id | string | 标准化报告ID | 去重，非空 |
| standard_sample_id | string | 标准化样品ID | 去重，非空 |
| standard_indicator_name | string | 统一映射后的标准指标名 | 枚举值校验，非空 |
| time_point | string | 检测时间点 | 非空，格式统一 |
| group_name | string | 组别名称 | 非空 |
| mean_value | double | 均值 | 保留原始小数位数，非空 |
| std_dev | double | 标准差 | 保留原始精度，允许空 |
| change_rate | double | 变化率（百分比） | 保留原始数值（如30.37） |
| p_value | double | P值 | 保留原始精度，允许空 |
| source_report_id | string | 来源原始报告ID | 可追溯至输入报告 |
| source_page | int | 来源页码 | 可追溯至原始PDF页码 |
| source_location | string | 来源位置描述 | 可追溯至具体表格行列 |
