terraform {
  backend "s3" {
    bucket = "terraform-lambdaclass"
    key    = "kraken_sequencer/terraform.tfstate"
    region = "us-west-2"
  }

  required_version = ">= 1.2.0"
}
