# 质检器

## 工作流程

1. 假设系统已经存在 `spec` 的 `base.md` 和 `questionnare_cleanning.md`
2. 委托人提供 `record` 的 `questionnare_raw.csv`
3. 系统生成 `schema` 的 `questionnare.json` 
4. 系统生成 ` plan` 的 `questionnnare_cleanning_plan.md`
5. 系统生成 `procecssor`的`questionnaire_cleanner.py`和`inspector`的`questionnaire_cleanning_inspector`
5. 系统运行 `questionniare_cleanning.py` 获得 `record` 的 `questioninare_cleanned.csv`
6. 系统生成 `manifest`的`questionnare_cleanning.md`
7. 系统打包生成 `dataset`的`questionnare_cleanning.zip`和`recipe`的`questionnaire_recipe.zip`
