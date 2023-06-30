.PHONY: init node install-cairo-native run-fibonacci

init:
	cd benchmark &&	pip install -r requirements.txt

bench:
	cd benchmark && export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && fab local

node:
#	./client 127.0.0.1:9006 --size 512 --rate 250 --timeout 1000
	cargo run --bin node --release -- -vvv run --keys ./benchmark/.node-$(N).json --committee ./benchmark/.committee.json --store .db-$(N) --parameters ./benchmark/.parameters.json

install-cairo-native:
	git clone https://github.com/lambdaclass/cairo_native.git
	brew install llvm@16
	brew install tmux
	cd cairo_native && scripts/fetch-corelibs.sh && export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && cargo build --release

run-fibonacci:
	export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && ./cairo_native/target/release/cli run -f fib::fib::main ./cairo_programs/fib.cairo --available-gas 900000000
