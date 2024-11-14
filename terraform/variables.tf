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
