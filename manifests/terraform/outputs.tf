output "bucket_name" {
  description = "OSS 桶名"
  value       = alicloud_oss_bucket.studio.bucket
}

output "bucket_domain" {
  description = "OSS 桶访问域名（CDN 回源地址）"
  value       = format("%s.%s", alicloud_oss_bucket.studio.bucket, alicloud_oss_bucket.studio.extranet_endpoint)
}

output "cdn_domain" {
  description = "CDN 加速域名"
  value       = alicloud_cdn_domain_new.studio.domain_name
}

output "cdn_cname" {
  description = "CDN CNAME，需在 DNS 中配置指向（当前已配置为 kunlunaq.com）"
  value       = alicloud_cdn_domain_new.studio.cname
}
