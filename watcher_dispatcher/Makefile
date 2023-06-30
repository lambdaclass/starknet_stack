run:
	mix deps.get
	mix assets.build
	iex -S mix phx.server
rust:
	cd ./native/watcher_dispatcher && cargo check
format:
	mix format && cd ./native/watcher_dispatcher && cargo fmt
