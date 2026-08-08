# 静态站点部署与运维 —— 关键操作记录（脱敏）

> 记录时间：2026-08-08 | 场景：qtcloud-data studio（`data.cloud.quanttide.com`）首次部署全流程
> 本文件为脱敏版：不包含任何 AccessKey / Secret / 账号 ID，仅记录操作路径与排障结论。

## 一、架构与链路

```
studio/* tag
   ↓ qtcloud-devops release publish（创建 tag + GitHub Release）
GitHub Actions deploy-studio.yml
   ↓ flutter build web → ossutil cp
OSS 桶（*-studio，公共读 + 静态网站托管）
   ↓ CDN 回源（域名 CNAME → *.kunlunaq.com，HTTPS 泛域名证书）
用户浏览器
```

## 二、关键操作

### 1. 发布新版本

```bash
qtcloud-devops release audit -v studio/v0.1.0-alpha.1   # 预检（版本号/CHANGELOG/工作区/标签冲突）
qtcloud-devops release publish -v studio/v0.1.0-alpha.1 -f -y   # -f：强制重建已存在 tag
```

- 预检常见失败项：`pubspec.yaml` 版本未对齐、`CHANGELOG.md` 缺条目、工作区有未提交变更
- tag 命名 `studio/*` 与 `deploy-studio.yml` 触发条件一致

**坑：Flutter Web 子路由刷新 404（SPA fallback）**

`usePathUrlStrategy()` 的 path 路由（如 `/transfer`）直接访问/刷新时，
静态托管按路径找物理文件 → 404 `NoSuchKey`。

解决（OSS 静态托管 ErrorDocument 指向 index.html）：

```bash
# ① 更新 OSS 静态托管：ErrorDocument → index.html（URL 不变，返回 index.html 内容）
#    注意：不要用 CDN error_page（302 会丢失路径，Flutter 路由失配）
PUT /?website  body: <WebsiteConfiguration><IndexDocument><Suffix>index.html</Suffix></IndexDocument>
                     <ErrorDocument><Key>index.html</Key></ErrorDocument></WebsiteConfiguration>
# ② 验证：curl -o /dev/null -w "%{http_code}" https://<domain>/transfer  → 404 但内容为 index.html
#    浏览器正常渲染，Flutter 路径路由接管
```

**坑：GitHub Release notes 为空**

`release publish` 会自动生成 Release notes，但它解析 CHANGELOG 时要求版本头格式为
`## [0.1.0-beta.1]`（**不带 scope 前缀**）。仓库现有格式 `## [studio/v0.1.0-beta.1]`
会导致「CHANGELOG 写入失败」且 **Release body 为空**（audit 的 CHANGELOG 检查仍会通过，
因为它是子串匹配）。

修复方式：发布后手动补 body：

```bash
gh release edit studio/v0.1.0-beta.1 --repo quanttide/qtcloud-data \
  --notes-file release_notes.md   # 内容从 src/studio/CHANGELOG.md 对应条目提取
```

### 2. 创建 OSS 桶并开放公共读（核心！）

**坑：新桶默认开启桶级 BlockPublicAccess（`BlockPublicAccess=true`），此时所有设置公共读的 API 都会被拒**
（报错形如 `Put public bucket acl is not allowed`，容易误判为"阿里云新规禁止"）。

正确顺序：

```bash
# ① 创建桶（默认私有）
aliyun oss mb oss://<bucket> --acl private

# ② 关闭桶级 BlockPublicAccess（必须，否则第③步报错）
#    通过 OSS REST API 签名直调：PUT /?publicAccessBlock
#    body: <PublicAccessBlockConfiguration><BlockPublicAccess>false</BlockPublicAccess></PublicAccessBlockConfiguration>

# ③ 设置公共读
aliyun oss set-acl oss://<bucket> public-read -b

# ④ 开启静态网站托管（让根路径 / 自动返回 index.html）
#    OSS REST API：PUT /?website
#    body: <WebsiteConfiguration><IndexDocument><Suffix>index.html</Suffix></IndexDocument>
#          <ErrorDocument><Key>error.html</Key></ErrorDocument></WebsiteConfiguration>

# ⑤ 验证
curl -s -o /dev/null -w "%{http_code}" https://<bucket>.oss-cn-hangzhou.aliyuncs.com/   # 期望 200
```

**参考事实**（组织内既有桶均为 公共读 + BPA=false）：`qtdata-studio`、`qtcloud-delib-studio`、`qtclass-studio`。

### 3. CDN 域名配置

```bash
# 查看域名详情（源站、CNAME、状态）
aliyun cdn DescribeCdnDomainDetail --DomainName <domain>

# 切换回源（源站为 OSS 私有桶时，系统自动附带 oss_auth 配置）
aliyun cdn ModifyCdnDomain --DomainName <domain> --Sources '[{"type":"oss","content":"<bucket>.oss-cn-hangzhou.aliyuncs.com","port":443,"priority":20}]'

# 刷新缓存
aliyun cdn RefreshObjectCaches --ObjectPath "https://<domain>/" --ObjectType Directory
```

注意：CDN 域名要求已备案（大陆节点）、DNS CNAME 指向 CDN 加速地址。

### 4. 重新触发部署（tag 已存在时）

```bash
git push origin :refs/tags/studio/v0.1.0-alpha.1   # 删除远端 tag
git push origin refs/tags/studio/v0.1.0-alpha.1    # 重建（触发 Actions）
```

GitHub Release 按 tag 名自动重新关联，无需重建 Release。

## 三、排障记录

| 现象 | 根因 | 解决 |
|------|------|------|
| `Put public bucket acl is not allowed` | **桶级 BlockPublicAccess=true**（新桶默认） | 关闭桶级 BPA 后重试（见上） |
| CDN 访问 403，OSS 报 `Anonymous user has no right` | 桶私有 + CDN 匿名回源 | 桶开放公共读（或配置私有回源签名） |
| 根路径 `/` 403，`index.html` 200 | 未开静态网站托管，空对象不可读 | 开启 `?website`，根路径自动映射 index.html |
| 签名直调 OSS 报 `SignatureDoesNotMatch` | CanonicalizedResource 桶级需带斜杠：`/{bucket}/?subresource`（不是 `/{bucket}?`） | 修正资源路径 |
| `x-oss-cdn-auth: success` 但仍 403 | 该头仅表示请求走了 CDN 私有回源通道；**success + 200 才是正常**，403 说明回源被桶权限拒绝 | 解决桶权限（见上） |
| 账号级 BPA 查询 | `GET /?publicAccessBlock`（二级域名）返回 `BlockPublicAccess=false` | 账号级未开启；问题在桶级 |

## 四、签名直调脚本（脱敏模板）

OSS REST API（数据面 V1 签名）通用模板。**凭据从本地 aliyun CLI 配置读取，勿硬编码**：

```python
import json, hmac, hashlib, base64, urllib.request, datetime

# 从 ~/.aliyun/config.json 读取默认 profile 的 AK/Secret（不在此展示）
with open('/home/<user>/.aliyun/config.json') as f:
    cfg = json.load(f)
profile = next(p for p in cfg['profiles'] if p.get('name') == 'default')
ak, secret = profile['access_key_id'], profile['access_key_secret']

def call(bucket, query, method='GET', body=None, content_type=None, content_md5=''):
    date = datetime.datetime.now(datetime.timezone.utc).strftime('%a, %d %b %Y %H:%M:%S GMT')
    resource = f'/{bucket}/?{query}'          # 注意桶级资源带尾部斜杠！
    string_to_sign = f"{method}\n{content_md5}\n{content_type or ''}\n{date}\n{resource}"
    sig = base64.b64encode(hmac.new(secret.encode(), string_to_sign.encode(), hashlib.sha1).digest()).decode()
    req = urllib.request.Request(f'https://{bucket}.oss-cn-hangzhou.aliyuncs.com/?{query}',
                                 data=body.encode() if body else None, method=method)
    req.add_header('Date', date)
    req.add_header('Authorization', f'OSS {ak}:{sig}')
    if content_type: req.add_header('Content-Type', content_type)
    if content_md5: req.add_header('Content-MD5', content_md5)
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            return resp.status, resp.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()

# 示例：查询桶级 BPA
print(call('<bucket>', 'publicAccessBlock'))
```

> ossutil 无 BPA/website 命令，这类操作需用上述 REST 直调方式。

## 五、检查清单

- [ ] 发布预检 `release audit` 全绿（版本/CHANGELOG/工作区）
- [ ] 桶：BPA=false + ACL public-read + 静态网站托管
- [ ] CDN：回源指向正确桶、`DescribeCdnDomainDetail` 显示 `online`
- [ ] `curl https://<domain>/` 返回 200
- [ ] 页面内容校验（`<title>` 等）
- [ ] 组织级运维手册：`quanttide-platform/docs/site-ops-handbook.md`

## 六、相关资源

- 部署工作流：`.github/workflows/deploy-studio.yml`
- 基础设施定义：`manifests/terraform/`（Terraform：桶 + CDN；含 BPA 踩坑注释）
- 组织运维手册：`quanttide-platform/docs/site-ops-handbook.md`
