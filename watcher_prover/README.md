# WatcherProver

## Description

This is the watcher - prover service for the proving system.

The architecture consists of a watcher thats monitors the blockchain for confirmed blocks and a prover that generate proofs for every transaction of the block. It could be plugged to any blockchain that supports smart contracts.

When the watcher finds a new transaction with a program, it first calls [cairo-rs](https://github.com/lambdaclass/cairo-rs/) to run the Cairo program and generate the trace. This trace is then sent to lambdaworks prover that creates the proof. The proof is then put in a later block in the blockchain.

At the beggining, this operations will be done sequentially. But in the future, the goal is to make them in parallel, scalling horizontally.

By running provers in parallel, throughput of the proving system will be as high as the throughput of the blockchain. However, there is a latency of the prover and the inclusion of the proofs in blocks.

## Configuration and environment variables

The following environment variables are required:

For the type of storage to use:

- `PROVER_STORAGE`:
  - `local` for storing the proof in the local filesystem (files will be stored in proofs/ directory) or
  - `s3` for storing the proof in an AWS S3 bucket

For AWS S3 bucket (used to upload the generated proofs):

- `AWS_ACCESS_KEY_ID`: AWS access key id
- `AWS_SECRET_ACCESS_KEY`: AWS secret access key
- `AWS_REGION`: AWS region

For the blockchain connection:

- `RPC_HOST`: RPC hostname of the node to connect to
- `RPC_PORT`: RPC port of the node to connect to

For example:

``` bash
RPC_HOST=localhost RPC_PORT=10009 make run
```
