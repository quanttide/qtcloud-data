# UV 迁移指南

## 从 Poetry 迁移到 UV

### 安装 UV

```bash
# macOS/Linux
curl -LsSf https://astral.sh/uv/install.sh | sh

# Windows
powershell -c "irm https://astral.sh/uv/install.ps1 | iex"

# 使用 pip
pip install uv
```

### 项目初始化

```bash
cd src/provider
# UV 会自动检测现有的 pyproject.toml
uv sync

# 安装开发依赖
uv sync --dev
```

### 常用命令对比

| 操作 | Poetry | UV |
|------|--------|-----|
| 安装依赖 | `poetry install` | `uv sync` |
| 安装开发依赖 | `poetry install --dev` | `uv sync --dev` |
| 添加依赖 | `poetry add fastapi` | `uv add fastapi` |
| 添加开发依赖 | `poetry add --dev pytest` | `uv add --dev pytest` |
| 更新依赖 | `poetry update` | `uv lock --upgrade-package` |
| 运行命令 | `poetry run pytest` | `uv run pytest` |
| 激活环境 | `poetry shell` | `source .venv/bin/activate` |

### 配置文件变更

#### Poetry (旧)
```toml
[build-system]
requires = ["poetry-core"]
build-backend = "poetry.core.masonry.api"

[tool.poetry]
name = "qtcloud-data-provider"
version = "0.1.0-alpha.1"

[tool.poetry.dependencies]
python = "^3.9"
fastapi = "^0.104.0"

[tool.poetry.group.dev.dependencies]
pytest = "^7.4.2"
```

#### UV (新)
```toml
[project]
name = "qtcloud-data-provider"
version = "0.1.0-alpha.1"
requires-python = ">=3.9"

dependencies = [
    "fastapi>=0.104.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.4.2",
]

[tool.uv]
dev-dependencies = [
    "pytest>=7.4.2",
]
```

### 主要区别

1. **语法更标准**：使用 PEP 621 标准，兼容性更好
2. **更快的速度**：UV 用 Rust 编写，比 Poetry 快 10-100 倍
3. **锁文件格式**：使用 `uv.lock` 替代 `poetry.lock`
4. **虚拟环境**：自动在 `.venv` 创建，位置统一

### 清理 Poetry

```bash
# 删除 Poetry 相关文件
rm -f poetry.lock
rm -f .python-version

# 如果完全移除 Poetry
pip uninstall poetry
```

### 测试验证

```bash
# 安装依赖
uv sync --dev

# 运行测试
uv run pytest

# 运行应用
uv run uvicorn app.main:app --reload
```

### 常见问题

**Q: 是否需要重写所有依赖？**
A: 不需要，UV 会自动解析现有的 `pyproject.toml`。

**Q: 锁文件可以提交到 Git 吗？**
A: 可以，`uv.lock` 应该提交以保证可重现性。

**Q: 如何处理本地包？**
A: 使用 `uv add -e ../my-package` 安装本地包。

**Q: CI/CD 如何配置？**
A: 确保先安装 UV，然后运行 `uv sync --dev`。

### 参考资源

- [UV 官方文档](https://github.com/astral-sh/uv)
- [PEP 621 - 储存 pyproject.toml 中的项目元数据](https://peps.python.org/pep-pep-0621/)
- [从 Poetry 迁移](https://github.com/astral-sh/uv?tab=readme-ov-file#from-poetry)
