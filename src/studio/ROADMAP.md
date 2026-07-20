# Studio ROADMAP

> 当前：无版本号（原型阶段）
> 目标版本 v0.1.0：Blueprint 页面

## v0.1.0 — Blueprint 页面

核心目标：让 Studio 能够展示和操作数据蓝图，覆盖列表浏览和详情查看。

### 页面规划

| 路由 | 页面 | 说明 |
|------|------|------|
| `/blueprints` | 蓝图列表 | 替换当前硬编码卡片，从 Provider API 拉取 |
| `/blueprints/:id` | 蓝图详情 | 三格式切换（.md / .cue / .html），含元数据 |

### 数据对接

| 数据 | 来源 | 说明 |
|------|------|------|
| 蓝图列表 | `GET /blueprints` | 蓝图名称、契约摘要、管道名称、验收规则数 |
| 蓝图详情 | `GET /blueprints/:id` | 完整蓝图内容（md/cue/html 三格式） |
| 蓝图元数据 | 同上 | 负责人、复核人、版本历史、对应仓库 |

### 交互流程

参考 CLI 五命令工作流，Studio 侧重"看"和"管"：

```
蓝图列表 → 点击进入详情 → 切换 md/cue/html → 查看版本历史
```

- v0.1.0 不做编辑，先打通"浏览"链路
- 后续版本逐步加入 review/design/formalize 操作

### 实现顺序

1. 蓝图列表页：Provider API 对接，替换硬编码数据
2. 蓝图详情页：路由参数 + 三格式 tab 切换
3. 版本历史组件：git-based version list
4. 错误/空态处理：API 不可用时的降级展示

### 依赖

- Provider 需实现 `GET /blueprints` 和 `GET /blueprints/:id` 端点
- Provider 待实现时，Studio 先 mock 数据开发

### 交付标准

- 蓝图列表从 API 拉取真实数据
- 蓝图详情页三格式可切换
- 页面在 Web/Linux 两端可运行
- 雅芳/瑜娇/董朗验收通过
