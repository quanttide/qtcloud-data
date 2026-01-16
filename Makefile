.PHONY: help install test test-fixtures test-provider test-sdk test-studio clean

help:  ## 显示所有可用命令
	@echo "可用命令:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

install:  ## 安装所有依赖
	@echo "安装依赖..."
	@python -m pip install --upgrade pip
	@python -m pip install pytest pandas

test:  ## 运行所有测试
	@echo "运行所有测试..."
	@python3 -m pytest tests/ -v

test-fixtures:  ## 运行 fixtures 验证测试
	@echo "运行 fixtures 验证测试..."
	@python3 -m pytest tests/test_questionnaire_cleanning.py -v

test-fixtures-structure:  ## 只测试 fixtures 目录结构
	@echo "测试 fixtures 目录结构..."
	@python3 -m pytest tests/test_questionnaire_cleanning.py::TestFixturesStructure -v

test-plan:  ## 只测试 Plan 文件
	@echo "测试 Plan 文件..."
	@python3 -m pytest tests/test_questionnaire_cleanning.py::TestPlan -v

test-schema:  ## 只测试 Schema 文件
	@echo "测试 Schema 文件..."
	@python3 -m pytest tests/test_questionnaire_cleanning.py::TestSchema -v

test-data:  ## 只测试数据记录
	@echo "测试数据记录..."
	@python3 -m pytest tests/test_questionnaire_cleanning.py::TestDataRecords -v

test-inspector:  ## 只测试 Inspector 功能
	@echo "测试 Inspector 功能..."
	@python3 -m pytest tests/test_questionnaire_cleanning.py::TestInspector -v

test-manifest:  ## 只测试 Manifest 清单
	@echo "测试 Manifest 清单..."
	@python3 -m pytest tests/test_questionnaire_cleanning.py::TestManifest -v

test-consistency:  ## 只测试数据一致性
	@echo "测试数据一致性..."
	@python3 -m pytest tests/test_questionnaire_cleanning.py::TestDataConsistency -v

test-transformations:  ## 只测试数据转换规则
	@echo "测试数据转换规则..."
	@python3 -m pytest tests/test_questionnaire_cleanning.py::TestTransformations -v

test-report:  ## 只测试报告文件
	@echo "测试报告文件..."
	@python3 -m pytest tests/test_questionnaire_cleanning.py::TestReport -v

test-provider:  ## 运行 Provider 测试
	@echo "运行 Provider 测试..."
	@if [ -d "src/provider" ]; then \
		cd src/provider && uv run pytest; \
	else \
		echo "Provider 目录不存在"; \
	fi

test-sdk:  ## 运行 Python SDK 测试
	@echo "运行 Python SDK 测试..."
	@if [ -d "src/python_sdk" ]; then \
		cd src/python_sdk && uv run pytest; \
	else \
		echo "Python SDK 目录不存在"; \
	fi

test-studio:  ## 运行 Studio 测试
	@echo "运行 Studio 测试..."
	@if [ -d "src/studio" ]; then \
		cd src/studio && flutter test; \
	else \
		echo "Studio 目录不存在"; \
	fi

test-coverage:  ## 运行测试并生成覆盖率报告
	@echo "运行测试并生成覆盖率报告..."
	@python3 -m pytest tests/ --cov=tests --cov-report=html --cov-report=term

clean:  ## 清理临时文件
	@echo "清理临时文件..."
	@find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
	@find . -type d -name ".pytest_cache" -exec rm -rf {} + 2>/dev/null || true
	@find . -type f -name "*.pyc" -delete 2>/dev/null || true
	@find . -type f -name ".coverage" -delete 2>/dev/null || true
	@find . -type d -name "htmlcov" -exec rm -rf {} + 2>/dev/null || true
	@echo "清理完成"
