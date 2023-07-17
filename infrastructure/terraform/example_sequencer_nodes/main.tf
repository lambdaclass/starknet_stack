module "sequencer_cluster_1" {
  source = "../modules/aws_ec2_cluster"

  cluster_name  = "starknet_stack_sequencer_0"
  ssh_key_name  = "klaus"
  node_count    = 4
  instance_type = "c7g.xlarge"
}
