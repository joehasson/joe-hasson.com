terraform {
    required_providers {
        aws = {
          source  = "hashicorp/aws"
          version = "~> 5.0"
        }

        cloudflare = {
          source = "cloudflare/cloudflare"
          version = "~> 4.0"
        }
    }

    backend "s3" {
       bucket = "joe-hasson-portfolio-terraform-state"
       key    = "terraform.tfstate"
       region = "eu-west-2"
    }
}

provider "aws" {
  region = "eu-west-2"
}

provider "cloudflare" {
    api_token = var.cloudflare_api_token
}

# Create DNS record
resource "cloudflare_record" "api" {
  zone_id = var.cloudflare_zone_id  # Get this from Cloudflare dashboard
  name    = "@"  # Represents root domain
  value   = aws_eip.app_ip.public_ip
  type    = "A"
  proxied = true  # This enables Cloudflare's SSL and CDN
}

