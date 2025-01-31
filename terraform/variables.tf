variable "project_name" {
  description = "Name of the project"
  type        = string
  default     = "joe-hasson-site"
}

variable "environment" {
  description = "Environment (staging/prod)"
  type        = string
  default     = "staging"
}

variable "ssh_public_key" {
    description = "Public key to use for SSH access"
    type        = string
}

variable "cloudflare_api_token" {
  description = "Cloudflare API Token"
  sensitive   = true
}

variable "cloudflare_zone_id" {
    description = "Cloudflare Zone ID"
    sensitive = true
}

variable "deploy_machine_ip" {
    description = "IP address of machine used to run deployment"
    type        = string
}

