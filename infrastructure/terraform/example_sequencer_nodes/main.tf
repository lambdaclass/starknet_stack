module "sequencer_cluster_1" {
  source = "../modules/aws_ec2_cluster"

  cluster_name  = "starknet_stack_sequencer_0"
  ssh_key_name  = "klaus"
  node_count    = 4
  instance_type = "c6a.large"
}

module "sequencer_cluster_2" {
  source = "../modules/aws_ec2_cluster"

  cluster_name  = var.cluster_name
  ssh_key_name  = var.ssh_key_name
  node_count    = var.node_count
  instance_type = var.instance_type

  aws_ami_id = var.aws_ami_id
}

variable "cluster_name" {}
variable "ssh_key_name" {}
variable "node_count" {}
variable "instance_type" {}
variable "aws_ami_id" {}
