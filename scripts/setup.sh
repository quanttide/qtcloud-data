#!/bin/bash
# 项目初始化脚本
# 用法: ./scripts/setup.sh [provider|studio|all]

set -e

PROJECT_NAME=${1:-provider}
PROJECT_ROOT=$(cd "$(dirname "$0")/.." && pwd)

echo "🚀 量潮数据云 - 项目初始化"
echo ""

# 检查是否安装了 uv
if ! command -v uv &> /dev/null; then
    echo "📦 安装 UV..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.cargo/bin:$PATH"
fi

echo "✅ UV 已安装: $(uv --version)"
echo ""

# 根据项目类型执行不同的初始化逻辑
case $PROJECT_NAME in
    provider)
        echo "📦 初始化 Provider 项目..."
        cd "$PROJECT_ROOT/src/provider"
        uv sync --dev
        ;;

    studio)
        echo "📦 初始化 Studio 项目..."
        cd "$PROJECT_ROOT/src/studio"
        flutter pub get
        ;;

    all)
        echo "📦 初始化所有项目..."
        cd "$PROJECT_ROOT/src/provider"
        uv sync --dev

        cd "$PROJECT_ROOT/src/studio"
        flutter pub get
        ;;

    *)
        echo "❌ 未知的项目类型: $PROJECT_NAME"
        echo ""
        echo "用法: $0 [provider|studio|all]"
        exit 1
        ;;
esac

echo ""
echo "✅ 项目初始化完成！"
echo ""
echo "📌 常用命令:"
echo ""

case $PROJECT_NAME in
    provider)
        echo "   cd src/provider"
        echo "   uv run pytest                    # 运行测试"
        echo "   uv run pytest -v                 # 详细测试输出"
        echo "   uv run uvicorn app.main:app --reload  # 启动开发服务器"
        echo "   uv add <package>                 # 添加依赖"
        echo "   uv add --dev <package>           # 添加开发依赖"
        ;;

    studio)
        echo "   cd src/studio"
        echo "   flutter run                     # 运行 Flutter 应用"
        echo "   flutter test                    # 运行测试"
        echo "   flutter pub add <package>        # 添加依赖"
        ;;

    all)
        echo "Provider:"
        echo "   cd src/provider"
        echo "   uv run pytest"
        echo "   uv run uvicorn app.main:app --reload"
        echo ""
        echo "Studio:"
        echo "   cd src/studio"
        echo "   flutter run"
        echo "   flutter test"
        ;;
esac

echo ""
