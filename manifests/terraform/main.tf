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
  # 关键经验（2026-08-08 踩坑记录）：
  # 1. 新桶默认开启【桶级 BlockPublicAccess】→ 需先关闭才能设置公共读
  #    （关闭方式：PUT /{bucket}?publicAccessBlock 设置 BlockPublicAccess=false，
  #     其他应用桶 qtdata-studio / qtcloud-delib-studio / qtclass-studio 均为 false）
  # 2. 关闭后设置 acl = public-read（阿里云禁止的是 API 设置，关闭 BPA 后允许）
  # 3. 开启静态网站托管，使根路径 / 自动返回 index.html
  acl = "public-read"

  website {
    index_document = "index.html"
    error_document  = "error.html"
  }

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
