# 量潮数据云

## 项目结构

```
qtcloud-data/
├── src/
│   ├── provider/          # 后端服务（FastAPI）
│   ├── cli/              # 命令行工具
│   └── studio/           # 前端 Flutter 应用
├── tests/                # 测试fixtures和数据
├── docs/                 # 文档
└── scripts/              # 项目初始化脚本
```

## 快速开始

### 前置要求

- Python >= 3.9
- [UV](https://github.com/astral-sh/uv) - Python 包管理器（推荐）
- Flutter >= 3.0（仅 Studio）

### 安装 UV

```bash
# macOS/Linux
curl -LsSf https://astral.sh/uv/install.sh | sh

# Windows
powershell -c "irm https://astral.sh/uv/install.ps1 | iex"

# 或使用 pip
pip install uv
```

### 项目初始化

使用顶层的初始化脚本来设置开发环境：

```bash
# 初始化所有项目
./scripts/setup.sh all

# 或者单独初始化某个项目
./scripts/setup.sh provider
./scripts/setup.sh studio
```

### 常用命令

#### Provider（后端服务）

```bash
cd src/provider

# 安装依赖
uv sync --dev

# 运行测试
uv run pytest
uv run pytest -v  # 详细输出

# 启动开发服务器
uv run uvicorn app.main:app --reload

# 添加依赖
uv add <package>
uv add --dev <package>
```

#### Studio（前端应用）

```bash
cd src/studio

# 安装依赖
flutter pub get

# 运行应用
flutter run

# 运行测试
flutter test

# 添加依赖
flutter pub add <package>
```

## 开发工作流

1. **克隆仓库**
   ```bash
   git clone <repository-url>
   cd qtcloud-data
   ```

2. **初始化环境**
   ```bash
   ./scripts/setup.sh all
   ```

3. **启动开发服务器**
   ```bash
   # Provider
   cd src/provider
   uv run uvicorn app.main:app --reload

   # Studio
   cd src/studio
   flutter run
   ```

4. **运行测试**
   ```bash
   # Provider & Python SDK
   uv run pytest

   # Studio
   flutter test
   ```

## License

Apache 2.0
