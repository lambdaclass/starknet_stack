# Starknet Stack

`````mermaid
flowchart LR
	A("Client") ==>|"Starknet Transactions"| subGraph0["Sequencer"]
	subGraph0 -.->|"Blocks with txs"| 300319["Watcher prover"]
	300319 ==>|"Request blocks through RPC "| subGraph0
	300319 ==>|"STARK proofs"| 495216[("Proof Storage\n")]
	style 495216 stroke-dasharray: 5
	subgraph 300319["Watcher prover"]
		320311("Cairo VM") ==>|"trace"| 993791("Lambdaworks Prover")
	end
	subgraph subGraph0["Sequencer"]
		C("Consensus") ==x|"tx settlement"| B("Cairo Native")
		B -.->|"tx execution info"| C
	end
`````

## Overview

Starknet stack is a set of technologies to launch and run high-performance decentralized validity blockchains based on Starknet and Cairo. It encompasses the whole cycle: Sequencing user transactions into blocks, executing them, and generating validity proofs, in order to settle state transitions while maintaining high throughput and transparency.

There are two main components to the cycle:

- [Sequencer](/sequencer): The sequencing side of the flow written in Rust, which includes user transaction settlement and execution through [Cairo Native](https://github.com/lambdaclass/cairo_native).
- [Watcher-Prover](/watcher-prover): A service that can be deployed independently which is in charge of requesting blocks with transactions to the sequencer nodes, in order to get transactions and generate traces with [Cairo VM](https://github.com/lambdaclass/cairo-vm/) which are later proved by our [Lambdaworks Starknet Prover](https://github.com/lambdaclass/starknet_stack_prover_lambdaworks). The proofs are later stored for users to query them accordingly.

## Quick start



