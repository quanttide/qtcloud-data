# 问卷数据清洗

## 测试流程

1. 假设系统已经存在 `spec` 的 `base.md` 和 `questionnare_cleanning.md`
2. 委托人提供 `record` 的 `questionnare_raw.csv`
3. 代理人提供 `blueprint` 的 `questionnnare_cleanning`
4. 系统生成 `schema` 的 `questionnare.json` 和 `procecssor`的`questionnare_cleanner.py`
5. 系统运行 `questionnare_cleanning.py` 获得 `record` 的 `questionnare_cleanned.csv`
6. 系统生成 `manifest`的`questionnare_cleanning.md`
7. 系统打包生成 `dataset`的`questionnare_cleanning.zip`
