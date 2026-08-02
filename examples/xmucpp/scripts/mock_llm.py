#!/usr/bin/env python3
"""Mock LLM server（OpenAI 兼容 /chat/completions）：按 prompt 关键词返回预设响应。
演示 qtcloud-data 生命周期命令的 LLM 可注入架构，无需真实 API key。"""
import json
from http.server import HTTPServer, BaseHTTPRequestHandler

DRD = """# 电商价格数据库

## 需求背景
两个电商平台（A/B），每天采集一次商品价格与销量，持续采集 365 天。
根据固定关键词列表进入搜索页获取商品列表及价格、销量，并进入商品详情页获取商品规格。

## 数据范围
- 平台：2 个
- 采集频率：每日
- 采集周期：365 天
- 采集对象：关键词列表对应商品的 价格 / 销量 / 规格

## 数据质量要求
- 缺失值不能高于 30%（数据契约）
- 价格缺失处理：单天缺失且前后两天数据未变时，非促销日假设缺失值等于前值（可接受）；
  促销日缺失视为违背数据契约
"""

CONTRACT_TABLES = """## 输入契约
| 字段名 | 类型 | 描述 |
| --- | --- | --- |
| product_id | string | 商品ID |
| platform | string | 电商平台 |
| keyword | string | 搜索关键词 |
| price | number | 采集到的价格 |
| sales | number | 采集到的销量 |

## 输出契约
| 字段名 | 类型 | 描述 |
| --- | --- | --- |
| product_id | string | 商品ID |
| platform | string | 电商平台 |
| date | string | 采集日期 |
| price | number | 商品价格 |
| sales | number | 商品销量 |
| missing | boolean | 当日数据是否缺失 |
"""

BLUEPRINT_TABLES = """## 处理步骤
| 步骤名 | 从 | 到 | 描述 | 依赖 |
| --- | --- | --- | --- | --- |
| categorize | raw_records | categorized | 商品类别分配器：按关键词分类 | - |
| collect_list | search_page | product_list | 商品列表采集器：搜索页抓取列表 | categorize |
| collect_detail | product_detail | product_records | 商品详情采集器：详情页抓取规格 | collect_list |
"""

PYTHON_STEP = """```python
def categorize(data):
    \"\"\"商品类别分配器：按关键词给商品分类\"\"\"
    import pandas as pd
    df = pd.read_csv(data)
    df["category"] = df["keyword"]
    return df
```
"""

PYTHON_ASSEMBLE = """```python
import pandas as pd
import argparse

def categorize(data):
    \"\"\"商品类别分配器：按关键词给商品分类\"\"\"
    df = pd.read_csv(data)
    df["category"] = df["keyword"]
    return df

def collect_list(data):
    \"\"\"商品列表采集器：搜索页抓取商品列表\"\"\"
    return data

def collect_detail(data):
    \"\"\"商品详情采集器：详情页抓取规格\"\"\"
    return data

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("input")
    parser.add_argument("output")
    args = parser.parse_args()
    result = categorize(args.input)
    result = collect_list(result)
    result = collect_detail(result)
    result.to_csv(args.output, index=False)
```
"""

class Handler(BaseHTTPRequestHandler):
    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("Content-Length", 0)))
        req = json.loads(body)
        prompt = req["messages"][-1]["content"]
        if "需求分析师" in prompt:
            content = DRD
        elif "生成数据契约" in prompt:
            content = CONTRACT_TABLES
        elif "生成处理蓝图" in prompt:
            content = BLUEPRINT_TABLES
        elif "组装" in prompt:
            content = PYTHON_ASSEMBLE
        else:
            content = PYTHON_STEP
        data = json.dumps({"choices": [{"message": {"content": content}}]}).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def log_message(self, *args):
        pass

HTTPServer(("127.0.0.1", 8899), Handler).serve_forever()
