data "aws_ami" "debian_11_latest_arm64" {
  most_recent = true

  owners = ["136693071363"]

  filter {
    name = "name"
    values = ["debian-11-arm64-*"]
  }
}

data "aws_vpc" "starknet_stack_sequencer" {
  filter {
    name   = "tag:Name"
    values = ["starknet_stack_sequencer_0-vpc"]
  }
}

data "aws_subnet" "sequencer" {
  filter {
    name   = "vpc-id"
    values = [data.aws_vpc.starknet_stack_sequencer.id]
  }
}

data "aws_security_group" "starknet_sequencer" {
  name = "starknet_stack_sequencer_0_nodes_sg"
}

resource "aws_security_group" "watcher_prover" {
  name        = "watcher_prover"
  vpc_id      = data.aws_vpc.starknet_stack_sequencer.id
}

resource "aws_security_group_rule" "outbound_traffic" {
  security_group_id = aws_security_group.watcher_prover.id

  description      = "Allow all outbound traffic"
  type             = "egress"
  from_port        = 0
  to_port          = 0
  protocol         = "-1"
  cidr_blocks      = ["0.0.0.0/0"]
  ipv6_cidr_blocks = ["::/0"]
}

variable "ssh_allowed_ips" {
  type        = list
  description = "List of CIDR blocks to allow SSH access to the watcher_prover"
}

resource "aws_security_group_rule" "ssh" {
  security_group_id = aws_security_group.watcher_prover.id

  description = "Allow SSH access"
  type        = "ingress"
  from_port   = 22
  to_port     = 22
  protocol    = "tcp"
  cidr_blocks = var.ssh_allowed_ips
}

resource "aws_security_group_rule" "internal_traffic" {
  security_group_id = aws_security_group.watcher_prover.id

  description = "Allow all internal traffic"
  type        = "ingress"
  from_port   = 0
  to_port     = 0
  protocol    = "-1"

  source_security_group_id = aws_security_group.watcher_prover.id
}

resource "aws_security_group_rule" "starknet_sequencer_to_watcher_prover" {
  security_group_id = aws_security_group.watcher_prover.id

  description = "Allow all traffic going from the starknet sequencer to the watcher_prover"
  type        = "ingress"
  from_port   = 0
  to_port     = 0
  protocol    = "-1"

  source_security_group_id = data.aws_security_group.starknet_sequencer.id
}

resource "aws_security_group_rule" "watcher_prover_to_starknet_sequencer" {
  security_group_id = data.aws_security_group.starknet_sequencer.id

  description = "Allow all traffic going from the watcher_prover to the starknet sequencer"
  type        = "ingress"
  from_port   = 0
  to_port     = 0
  protocol    = "-1"

  source_security_group_id = aws_security_group.watcher_prover.id
}

resource "aws_instance" "watcher_prover" {
  ami           = "${data.aws_ami.debian_11_latest_arm64.id}"
  instance_type = "m7g.xlarge"
  key_name      = "klaus"

  subnet_id              = "subnet-0b86c41d31a517d60"
  vpc_security_group_ids = [aws_security_group.watcher_prover.id]

  ebs_block_device {
    device_name = "/dev/xvda"
    volume_size = 200
  }

  tags = {
    Name = "watcher_prover"
  }
}

resource "aws_eip" "watcher_prover" {
  instance = "${aws_instance.watcher_prover.id}"
}
