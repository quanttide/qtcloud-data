# 量潮数据云

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