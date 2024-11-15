# Create DNS record
resource "cloudflare_dns_record" "root" {
  ttl = 1
  zone_id = var.cloudflare_zone_id
  name    = "@"  # Represents root domain
  content   = aws_eip.app_ip.public_ip
  type    = "A"
  proxied = true  # This enables Cloudflare's SSL and CDN
}
