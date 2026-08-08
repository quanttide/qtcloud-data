terraform {
  required_version = ">= 1.5"
  required_providers {
    alicloud = {
      source  = "aliyun/alicloud"
      version = "~> 1.240"
    }
  }
}

provider "alicloud" {
  region = var.region
}

# ============================================================
# OSS 桶：qtcloud-data studio Web 部署目标
# 与 .github/workflows/deploy-studio.yml 中的 oss:// 路径保持一致
# ============================================================
resource "alicloud_oss_bucket" "studio" {
  bucket = var.bucket_name
  # 注意：阿里云「公共读保护」新规禁止通过 API 将桶设为 public-read
  # （ACL / bucket policy / 对象级 ACL 均被拒绝）。
  # 桶保持私有；线上访问需在 CDN 控制台开启「私有回源」
  # （自动创建服务关联角色 AliyunServiceRoleForCdn 后签名回源），
  # 或在 OSS 控制台手动设置公共读（有风险确认流程，qtdata-studio 即如此）。
  acl = "private"

  tags = {
    App = "qtcloud-data-studio"
    Env = "production"
  }
}

# ============================================================
# CDN 域名：data.cloud.quanttide.com
# 前置条件：
#   1. 域名已在阿里云 CDN 完成接入（DNS CNAME 已指向 kunlunaq.com）
#   2. 大陆节点需要 ICP 备案；未备案请使用 scope = "overseas"
# ============================================================
resource "alicloud_cdn_domain_new" "studio" {
  domain_name = var.cdn_domain
  cdn_type    = "web"
  scope       = var.cdn_scope

  sources {
    type     = "oss"
    content  = format("%s.%s", alicloud_oss_bucket.studio.bucket, alicloud_oss_bucket.studio.extranet_endpoint)
    port     = 80
    priority = 20
  }
}
