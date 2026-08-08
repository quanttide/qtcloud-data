# qtcloud-data studio 基础设施

Terraform 管理「量潮数据云控制台」的云上基础设施：

- **OSS 桶** `qtcloud-data-studio`：studio Web 构建产物的部署目标（`deploy-studio.yml` 上传路径）
- **CDN 域名** `data.cloud.quanttide.com`：web 加速，回源到上述 OSS 桶

## 使用

前置要求：`terraform` ≥ 1.5，阿里云 AccessKey（具备 OSS / CDN 管理权限）。

```bash
cd manifests/terraform
cp terraform.tfvars.example terraform.tfvars   # 按需修改
export ALICLOUD_ACCESS_KEY_ID=xxx
export ALICLOUD_ACCESS_KEY_SECRET=xxx

terraform init
terraform plan
terraform apply
```

## 前置条件

| 项 | 说明 |
|----|------|
| DNS | `data.cloud.quanttide.com` 需 CNAME 到 CDN 分配的地址（当前指向 `*.kunlunaq.com`，无需变更） |
| ICP 备案 | 大陆 CDN 节点要求备案；如未备案，将 `cdn_scope` 设为 `overseas` |
| Secrets | GitHub Actions 部署需要仓库配置 `ALIYUN_ACCESS_KEY_ID` / `ALIYUN_ACCESS_KEY_SECRET` |

## 说明

- 桶 ACL 为 `public-read`（CDN 回源需要）；如需收紧为私有桶，请同步配置 CDN「OSS 私有回源」并保持 workflow 上传方式不变
- CDN 与桶的命名/区域与 `.github/workflows/deploy-studio.yml` 保持一致，避免部署失败
