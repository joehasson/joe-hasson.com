resource "aws_ebs_volume" "postgres_data" {
  availability_zone = aws_instance.app.availability_zone
  size             = 20
  type             = "gp3"
  
  tags = {
    Name = "postgres-data"
  }
}

resource "aws_volume_attachment" "postgres_data_att" {
  device_name = "/dev/xvdf"
  volume_id   = aws_ebs_volume.postgres_data.id
  instance_id = aws_instance.app.id
}

resource "aws_dlm_lifecycle_policy" "postgres_backup" {
  description        = "Weekly Postgres backups"
  execution_role_arn = aws_iam_role.dlm_lifecycle_role.arn
  state             = "ENABLED"

  policy_details {
    resource_types = ["VOLUME"]

    schedule {
      name = "Weekly backup"
      
      create_rule {
        interval      = 24
        times         = ["23:45"]
      }

      retain_rule {
        count = 3    # Keep last 3 snapshots
      }
    }

    target_tags = {
      Name = "postgres-data"
    }
  }
}
