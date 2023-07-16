.PHONY: init node install-cairo-native run-fibonacci

init: install-cairo-native
	rustup override set nightly-2023-06-19-aarch64-apple-darwin
	python3.9 -m venv ~/sequencer_venv && source ~/sequencer_venv/bin/activate && cd benchmark && pip install -r requirements.txt

bench:
	cd benchmark && export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && export RUST_LOG=info,salsa=off,cairo_native=off && fab local

N:=0
node: 
#	./client 127.0.0.1:9006 --size 512 --rate 250 --timeout 1000
	export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && cd benchmark && RUST_LOG=info,salsa=off,cairo_native=off,sled=off  cargo run --bin node --features benchmark -- -vvv run --keys ./.node-$(N).json --committee ./.committee.json --parameters ./.parameters.json --store ./.db-$(N)

reset:
	rm -rf ./benchmark/.db-*
	rm -rf ./.db-*

install-cairo-native:
	brew install llvm@16
	brew install tmux
	set -e
	git clone \
		--depth 1 \
		--branch v2.0.1 \
		https://github.com/starkware-libs/cairo.git \
		starkware-cairo
	cp -r starkware-cairo/corelib .
	rm -rf starkware-cairo/

build:
	export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && cargo build --all --release

test:
	export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && cargo test
