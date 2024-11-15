# Create DNS record
resource "cloudflare_dns_record" "root" {
  ttl = 1
  zone_id = var.cloudflare_zone_id
  name    = "@"  # Represents root domain
  content   = aws_eip.app_ip.public_ip
  type    = "A"
  proxied = true  # This enables Cloudflare's SSL and CDN
}

# Add cache rules
resource "cloudflare_page_rule" "cache_everything" {
  zone_id = var.cloudflare_zone_id
  targets = [{
    target = "url"
    constraint = {
      operator = "matches"
      value = "joe-hasson.com/*"
    }
  }]
  
  actions = [{
    cache_level = "cache_everything"
    edge_cache_ttl = 1800  # Cache for ~30 mins
  }]
}
