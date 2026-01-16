# 量潮数据云

## 项目结构

```
qtcloud-data/
├── src/
│   ├── provider/          # 后端服务（FastAPI）
│   ├── python_sdk/        # Python 工具箱
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
./scripts/setup.sh python_sdk
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

#### Python SDK

```bash
cd src/python_sdk

# 安装依赖
uv sync --dev

# 运行测试
uv run pytest
uv run pytest -v

# 添加依赖
uv add <package>
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

## 文档

- [Git 提交规范](#git-提交规范)
- [UV 迁移指南](docs/guides/uv-migration.md)

## Git 提交规范

为保证提交历史的一致性和可读性，所有 Git 提交消息必须遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范，格式如下：

```
<type>(<scope>): <description>
```

### 字段说明

- `<type>`: 提交类型
  - `feat`: 新功能(feature)
  - `fix`: 修复bug
  - `docs`: 文档(documentation)
  - `style`: 代码格式调整，不影响逻辑
  - `refactor`: 重构，既不修复bug也不添加功能
  - `test`: 测试相关
  - `chore`: 构建过程或辅助工具的变动
  - `init`: 初始化项目

- `<scope>`: 影响范围(可选)
  - `provider`: 后端服务
  - `studio`: 前端Flutter应用
  - `python_sdk`: Python SDK
  - `qtcloud-data`: 整个项目
  - `dataset`: 数据集相关
  - 其他特定模块名

- `<description>`: 简短描述
  - 使用祈使句、现在时态，例如"use"而非"used"或"uses"
  - 不要大写首字母
  - 不要以句号结尾

### 配置提交模板

为确保每次提交都遵循规范，请设置 Git 提交模板：

```bash
# 在项目根目录下执行
git config commit.template .gitmessage
```

### 示例

```text
feat(provider): 添加用户认证功能
docs(readme): 修正拼写错误
fix(dataset): 解决数据集查询的性能问题
chore: 更新构建脚本
```

### 历史问题

我们发现了一些不符合规范的历史提交，例如：

- `chore: add src/provider/` - 缺少scope部分
- `chore: add src/studio/` - 缺少scope部分
- `docs: 分离PRD和Guides` - 缺少scope部分
- `init: recreate project` - 缺少scope部分

请确保所有新的提交都遵循规范格式。

## License

Apache 2.0
