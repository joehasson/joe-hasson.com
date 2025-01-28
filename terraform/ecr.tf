locals {
  cleanup_policy = jsonencode({
    rules = [
      {
        rulePriority = 1
        description  = "Keep only latest image with tag 'latest'"
        selection = {
          tagStatus = "tagged"
          tagPrefixList = ["latest"]
          countType = "imageCountMoreThan"
          countNumber = 1
        }
        action = {
          type = "expire"
        }
      },
      {
        rulePriority = 2
        description  = "Remove all untagged images"
        selection = {
          tagStatus = "untagged"
          countType = "sinceImagePushed"
          countUnit = "days"
          countNumber = 1
        }
        action = {
          type = "expire"
        }
      }
    ]
  })
}

resource "aws_ecr_repository" "repositories" {
  for_each = toset(["backend", "reverse-proxy", "blog-post-dispatcher", "migrations"])

  name                 = each.key
  image_tag_mutability = "MUTABLE"
  force_delete         = true
}

resource "aws_ecr_lifecycle_policy" "cleanup" {
  for_each      = aws_ecr_repository.repositories

  repository = each.value.name
  policy     = local.cleanup_policy
}
