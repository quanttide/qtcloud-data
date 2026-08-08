variable "region" {
  description = "阿里云地域"
  type        = string
  default     = "cn-hangzhou"
}

variable "bucket_name" {
  description = "studio Web 部署的 OSS 桶名（与 deploy-studio.yml 保持一致）"
  type        = string
  default     = "qtcloud-data-studio"
}

variable "cdn_domain" {
  description = "studio CDN 加速域名"
  type        = string
  default     = "data.cloud.quanttide.com"
}

variable "cdn_scope" {
  description = "CDN 加速区域：domestic / overseas / global"
  type        = string
  default     = "global"
}
