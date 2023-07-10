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

