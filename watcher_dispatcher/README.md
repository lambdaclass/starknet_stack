# WatcherDispatcher

## Description

This is the watcher - dispatcher service for the proving system.

The architecture consists of a watcher thats monitors the blockchain for confirmed blocks and a dispatcher that sends these blocks to the provers. It could be plugged to any blockchain that supports smart contracts.

When the watcher finds a new transaction with a program, it first calls [cairo-rs](https://github.com/lambdaclass/cairo-rs/) to run the Cairo program and generate the trace. This trace is then sent to lambdaworks prover that creates the proof. The proof is then put in a later block in the blockchain.

At the beggining, this operations will be done sequentially. But in the future, the goal is to make them in parallel, scalling horizontally.

By running provers in parallel, throughput of the proving system will be as high as the throughput of the blockchain. However, there is a latency of the prover and the inclusion of the proofs in blocks.

## Installation and usage

To start your Phoenix server:

  * Run `mix setup` to install and setup dependencies
  * Start Phoenix endpoint with `mix phx.server` or inside IEx with `iex -S mix phx.server`

Now you can visit [`localhost:4000`](http://localhost:4000) from your browser.

Ready to run in production? Please [check our deployment guides](https://hexdocs.pm/phoenix/deployment.html).

## Learn more

  * Official website: https://www.phoenixframework.org/
  * Guides: https://hexdocs.pm/phoenix/overview.html
  * Docs: https://hexdocs.pm/phoenix
  * Forum: https://elixirforum.com/c/phoenix-forum
  * Source: https://github.com/phoenixframework/phoenix