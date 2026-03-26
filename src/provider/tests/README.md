# Usage

1. 将脱敏后的问卷数据（5-10条记录）放入 fixtures/raw/sample_questionnaire.xlsx
2. 将清洗后的期望结果放入 fixtures/expected/expected_cleaned.csv
3. 运行
```
cd tests
pytest test_cleaning.py -v
```

