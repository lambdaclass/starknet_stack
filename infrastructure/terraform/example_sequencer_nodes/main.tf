module "sequencer_cluster_1" {
  source = "../modules/aws_ec2_cluster"

  cluster_name = "kraken_sequencer_1"
  ssh_key_name = "klaus"
  node_count = 4
}
