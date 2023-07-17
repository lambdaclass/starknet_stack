data "aws_ami" "debian_11_latest" {
  most_recent = true

  owners = ["136693071363"]

  filter {
    name = "name"
    values = ["debian-11-arm64-*"]
  }
}

resource "aws_launch_template" "nodes" {
  name = "${var.cluster_name}_nodes_lt"

  block_device_mappings {
    device_name = "/dev/xvda"

    ebs {
      volume_size = 120
    }
  }

  network_interfaces {
    security_groups = [aws_security_group.nodes.id]
    associate_public_ip_address = true
    subnet_id = aws_subnet.us-west-2a.id
  }

  instance_type = var.instance_type
  image_id = data.aws_ami.debian_11_latest.id

  key_name = var.ssh_key_name

  iam_instance_profile {
    name = aws_iam_instance_profile.node.id
  }
}

resource "aws_autoscaling_group" "nodes" {
  name = "${var.cluster_name}_nodes"

  desired_capacity   = var.node_count
  max_size           = var.node_count
  min_size           = var.node_count

  availability_zones = ["us-west-2a"]

  launch_template {
    id      = aws_launch_template.nodes.id
    version = "$Latest"
  }

  tag {
    key = "Application"
    value = "${var.cluster_name}"
    propagate_at_launch = true
  }
}
