#!/bin/bash
# Provider 项目初始化脚本

set -e

echo "🚀 量潮数据云 Provider - 项目初始化"
echo ""

# 检查是否安装了 uv
if ! command -v uv &> /dev/null; then
    echo "📦 安装 UV..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.cargo/bin:$PATH"
fi

echo "✅ UV 已安装: $(uv --version)"
echo ""

# 同步依赖
echo "📥 同步依赖..."
uv sync --dev

echo ""
echo "✅ 项目初始化完成！"
echo ""
echo "📌 常用命令:"
echo "   uv run pytest              # 运行测试"
echo "   uv run pytest -v           # 详细测试输出"
echo "   uv run uvicorn app.main:app --reload  # 启动开发服务器"
echo "   uv add <package>          # 添加依赖"
echo "   uv add --dev <package>    # 添加开发依赖"
echo ""
