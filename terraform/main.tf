# main.tf
# VPC and Network Configuration
resource "aws_vpc" "main" {
  cidr_block           = "10.0.0.0/16"
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name = "${var.project_name}-vpc"
  }
}

# Public Subnet
resource "aws_subnet" "public" {
  vpc_id                  = aws_vpc.main.id
  cidr_block             = "10.0.1.0/24"
  availability_zone       = "eu-west-2a"
  map_public_ip_on_launch = true

  tags = {
    Name = "${var.project_name}-public-subnet"
  }
}

# Internet Gateway
resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id

  tags = {
    Name = "${var.project_name}-igw"
  }
}

# Route Table
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }

  tags = {
    Name = "${var.project_name}-public-rt"
  }
}

resource "aws_route_table_association" "public" {
  subnet_id      = aws_subnet.public.id
  route_table_id = aws_route_table.public.id
}

# Security Group
data "http" "cloudflare_ips_v4" {
  url = "https://api.cloudflare.com/client/v4/ips"
}

locals {
  cloudflare_ip_ranges = jsondecode(data.http.cloudflare_ips_v4.response_body).result.ipv4_cidrs
}

resource "aws_security_group" "app" {
  name        = "${var.project_name}-sg"
  description = "Security group for portfolio app"
  vpc_id      = aws_vpc.main.id

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]  # Consider restricting to your IP
  }

  # http port
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = local.cloudflare_ip_ranges
  }

  # https port
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = local.cloudflare_ip_ranges
  }

  # allow all egress
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name = "${var.project_name}-sg"
  }
}

# EC2 Instance with ssh key
resource "aws_key_pair" "deployer" {
  key_name   = "${var.project_name}-key"
  public_key = var.ssh_public_key
}


resource "aws_instance" "app" {
  ami           = "ami-0b2d89eba83fd3ed9"  # Amazon Linux 2 AMI
  instance_type = "t3.micro"
  subnet_id     = aws_subnet.public.id
  iam_instance_profile = aws_iam_instance_profile.ec2_profile.name
  
  vpc_security_group_ids = [aws_security_group.app.id]
  
  key_name = aws_key_pair.deployer.key_name

  root_block_device {
    volume_size = 20  # Size in GB
    volume_type = "gp3"  # Using gp3 as it's more cost-effective than gp2
    encrypted   = true
  }

  tags = {
    Name = "${var.project_name}-instance"
  }
}

resource "aws_eip" "app_ip" {
  instance = aws_instance.app.id
  domain   = "vpc"
  
  tags = {
    Name = "${var.project_name}-eip"
  }
}

# SSL Certificate
resource "aws_acm_certificate" "cert" {
  domain_name               = "joe-hasson.com"
  validation_method         = "DNS"

  lifecycle {
    create_before_destroy = true
  }
}
