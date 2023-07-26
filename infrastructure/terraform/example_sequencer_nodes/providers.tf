terraform {
  backend "s3" {
    bucket = var.terraform_aws_s3_bucket
    key    = "${ var.terraform_aws_s3_directory }/terraform.tfstate"
    region = "us-west-2"
  }

  required_version = ">= 1.2.0"
}

variable "terraform_aws_s3_bucket" {}
variable "terraform_aws_s3_directory" {}
