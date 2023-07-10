resource "aws_iam_policy" "releases_bucket" {
  count       = var.releases_s3 ? 1 : 0
  name        = "${var.cluster_name}-releases-bucket"
  path        = "/"
  description = "Allow nodes to pull binaries"

  policy = jsonencode({
    "Version" : "2012-10-17",
    "Statement" : [
      {
        "Sid" : "VisualEditor0",
        "Effect" : "Allow",
        "Action" : [
          "s3:GetObject",
          "s3:ListBucket",
        ],
        "Resource" : [
          "arn:aws:s3:::${var.releases_s3_bucket_name}",
          "arn:aws:s3:::${var.releases_s3_bucket_name}/*"
        ]
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "releases_bucket" {
  count      = var.releases_s3 ? 1 : 0
  role       = aws_iam_role.node.name
  policy_arn = aws_iam_policy.releases_bucket[count.index].arn
}

resource "aws_iam_role" "node" {
  name = "${var.cluster_name}-node"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Sid    = ""
        Principal = {
          Service = "ec2.amazonaws.com"
        }
      },
    ]
  })
}

resource "aws_iam_instance_profile" "node" {
  name = "${var.cluster_name}-node"
  role = aws_iam_role.node.name
}
