output "public_ip" {
  value = aws_eip.app_ip.public_ip
  description = "The public IP of the app server"
}
