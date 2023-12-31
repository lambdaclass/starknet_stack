# Cloud infrastructure for Starknet Stack

In this directory is contained all the code to deploy different parts of the Startknet Stack.

Currently, automation is in place to deploy a N node sequencer cluster. It creates a whole isolated environment inside AWS: VPC + all necessary network resources + autoscaling group + security groups allowing transaction and RPC endpoints for 0.0.0.0/0

Watcher Prover terraform is also here but standalone one (still needs to be pointed to the sequencer afterwards)

Madara Explorer is still not automated.

## Deploy N Sequencer Nodes in AWS

Currently deploying in us-west-2a Availability Zone

### Prerequisites

1. AWS account
2. [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html) configured, or AWS keys environment vars (region: us-west-2)
    * either run `aws configure` and set your access keys, or
    * export [AWS environment variables](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-envvars.html) (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` and `AWS_DEFAULT_REGION`)
3. [Terraform v1.4+](https://developer.hashicorp.com/terraform/tutorials/aws-get-started/install-cli)
4. [Ansible v2.12+](https://docs.ansible.com/ansible/latest/installation_guide/intro_installation.html)
5. Terraform module instantiation
    * see [example](./terraform/example_sequencer_nodes/main.tf#L10-L27)
      * you can reuse this example as you don't have it in your terraform state
6. Makefile environment variables: They are set at the start of the [Makefile](./Makefile), modify them to fit your needs (the values in the current Makefile are sample, you MUST change them):
    * `NODE_COUNT`
      * number of nodes to deploy
    * `CLUSTER_NAME`
      * logical name of the cluster for AWS resources
      * must be the same as the terraform module name
    * `SSH_KEY_NAME`
      * SSH key name (previously created/uploaded to AWS us-west-2/Oregon region)
    * `INSTANCE_TYPE`
      * AWS instance type to use for the nodes (must be amd64/x86_64 architecture)
    * `SSH_KEY_DIR`
      * local directory where your SSH private key is stored (the one that corresponds with `SSH_KEY_NAME`)
    * `S3_BUCKET`
      * S3 bucket in which to store terraform state
    * `S3_DIR`
      * S3 directory (inside `S3_BUCKET`) to store the terraform state
    * `S3_REGION`
      * region for the `S3_BUCKET`

### Execute deployment

* To deploy from scratch, from latest commit of `main` branch of this repo, execute (in this directory):

```shell
make sequencer
```

This command generates all the needed metadata for the nodes based o the environment variables specified. Specifically, these are the items we need:

- `committee.json`: Contains all the info that all the nodes need to know about each other in terms of the application (public keys, endpoints, etc.)
- Sequencer nodes keys: (One file per node) Contains the secret key alongside its public key 
- Ansible inventory

After this is done, the ansible playbook is executed accordingly in order to replicate the desired configuration.

* To deploy from a precompiled version (from commit [`3e2a28f`](https://github.com/lambdaclass/starknet_stack/tree/3e2a28f/sequencer) of this repo), execute:

```shell
make sequencer-precompiled
```

* To destroy all sequencer AWS resources

```shell
make destroy-sequencer
```

To watch the sequencer logs you can do so by logging in via SSH (admin user) to a node and executing

```shell
journalctl --user -xefu sequencer
```

Note: remove the `f` flag to stop the tail/follow behaviour
