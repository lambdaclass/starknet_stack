variable "cluster_name" {
  type = string
  description = "Name for the cluster for AWS resources to have prefixed"
}

variable "instance_type" {
  type = string
  description = "AWS EC2 instance type for the nodes"
}

variable "ssh_key_name" {
  type = string
  description = "Existing AWS SSH key to add to the launched EC2 instances"
}

variable "node_count" {
  type = string
  description = "Number of nodes to launch"
}

variable "releases_s3" {
  type = bool
  description = "Whether the release/binary is downloaded from S3"
  default = false
}

variable "releases_s3_bucket_name" {
  type = string
  description = "Name of the S3 bucket to push and pull the appliation binary"
  default = ""
}
