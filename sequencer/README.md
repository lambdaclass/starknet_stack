# Starknet Kraken Sequencer
A Starknet decentralized sequencer implementation.

## Getting started

> **Note:** The current consensus code is based heavily on [Albert Sonnino's (asonnino)  implementation](https://github.com/asonnino/hotstuff/) which was research focused instead of application focused. Modifications were made by Lambda Class mainly on the `node` crate to allow for proccessing of transactions for committed blocks. 

The objective of this project is easily deploy an L2 decentralized sequencer for Starknet. This will be one of the multiple implementations that will follow the decentralized sequencer.

The sequencer can be broken down into (roughly) 3 interchangeable modules:

- Mempool (Narwhal) which stores received transactions
- Consensus (Bullshark, Tendermint, Hotstuff) which orders transactions stored by the mempool
- Execution Engine, which executes the transactions on the state machine. The OS is given by Starknet in Rust and execution is deferred to Cairo Native or Cairo-rs.

By interchangeable modules we mean that the underlying algorithm implementation in the Mempool communication, the Consensus protocol or the Execution Engine can be changed and configured.

Additionally, in order to maintain and persist state, there is a State module which implements [PhotonDB](https://github.com/photondb/photondb) in a first iteration.

## Quick Start

To run the Sequencer cairo_native is needed for execution. To install [cairo_native](https://github.com/lambdaclass/cairo_native) use the following make target:

```bash
make install-cairo-native
```

For the installation process **only works with MacOS** and it only works with **rust nightly** (overriden with make init). It also needs `python3.9` installed.

To deploy and benchmark a testbed of 4 nodes on your local machine, clone the repo and initialize it with the following make target:

```bash
make init
```
This will install the python dependencies in the `hotstuff/benchmark` dir.

You also need to install Clang (required by rocksdb) and [tmux](https://linuxize.com/post/getting-started-with-tmux/#installing-tmux) (which runs all nodes and clients in the background). Finally, run a local benchmark using fabric:

```bash
make bench
```

This command may take a long time the first time you run it (compiling rust code in `release` mode may be slow) and you can customize a number of benchmark parameters in `fabfile.py`. When the benchmark terminates, it displays a summary of the execution similarly to the one below.

```text
-----------------------------------------
 SUMMARY:
-----------------------------------------
 + CONFIG:
 Faults: 0 nodes
 Committee size: 4 nodes
 Input rate: 1,000 tx/s
 Transaction size: 512 B
 Execution time: 20 s

 Consensus timeout delay: 1,000 ms
 Consensus sync retry delay: 10,000 ms
 Mempool GC depth: 50 rounds
 Mempool sync retry delay: 5,000 ms
 Mempool sync retry nodes: 3 nodes
 Mempool batch size: 15,000 B
 Mempool max batch delay: 10 ms

 + RESULTS:
 Consensus TPS: 967 tx/s
 Consensus BPS: 495,294 B/s
 Consensus latency: 2 ms

 End-to-end TPS: 960 tx/s
 End-to-end BPS: 491,519 B/s
 End-to-end latency: 9 ms
-----------------------------------------
```

### Running a node

To run a node, you need to have a valid committee file and valid parameters (`make bench` will create a set of them according to the config) and then you can do `make node N={n}`, where `{n}` is the number of the node according to the configuration (for example, node 0 maps to `.db-0` and `.node-0.json` and `*-0.log` files).

Consensus will start when nodes can communicate with each other, which means you need to run all nodes. This in turn means all nodes need to share the same config and no nodes can be added to the network afteward.

### Querying a node

A node prvides an RPC endpoint that can be used to query it's state.

Examples:

Return latest block number:
```
curl -H "Content-Type: application/json" http://localhost:10008 -d '{"jsonrpc": "2.0","method": "starknet_blockNumber","params": [],"id": 1}'
{"jsonrpc":"2.0","result":1,"id":1}%
```

Return block with its transactions:
```
curl -H "Content-Type: application/json" http://localhost:10008 -d '{"jsonrpc": "2.0","method": "starknet_getBlockWithTxs","params": [{"Number": 1}],"id": 1}'
{"jsonrpc":"2.0","result":{"status":"ACCEPTED_ON_L2","block_hash":"ab7f32","parent_hash":"1250433","block_number":1,"new_root":"37f70fa9","timestamp":1688498274,"sequencer_address":"b7b3be","transactions":[{"type":"INVOKE","version":"0x1","transaction_hash":"72759bd7","max_fee":"55b0e2b","version":"0x1","signature":["af37b11"],"nonce":"2d7620a1","type":"INVOKE","sender_address":"5701712","calldata":["7bffa3"]}]},"id":1}%
```

_please note that current results are hardcoded_. It will be upgraded soon
## Next steps

To do

## Reference links

* [Starknet sequencer](https://www.starknet.io/de/posts/engineering/starknets-new-sequencer#:~:text=What%20does%20the%20sequencer%20do%3F)
* [Papyrus Starknet full node](https://medium.com/starkware/papyrus-an-open-source-starknet-full-node-396f7cd90202)
* [Pathfinder (eqlabs full node)](https://github.com/eqlabs/pathfinder)
* [Blockifier](https://github.com/starkware-libs/blockifier)

### Starknet
* [Starknet State](https://docs.starknet.io/documentation/architecture_and_concepts/State/starknet-state/)
* [Starknet architecture](https://david-barreto.com/starknets-architecture-review/)
* [Starknet transaction lifecycle](https://docs.starknet.io/documentation/architecture_and_concepts/Blocks/transaction-life-cycle/)
* [Cairo book](https://cairo-book.github.io/title-page.html)
* [Starknet book](https://book.starknet.io/)
* [Starknet RPC specs](https://github.com/starkware-libs/starknet-specs)

### Other implementations
* [Madara](https://github.com/keep-starknet-strange/madara)
