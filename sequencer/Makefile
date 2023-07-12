.PHONY: init node install-cairo-native run-fibonacci

init:
	rustup toolchain install nightly-2023-06-19 --component rustc,cargo,rustfmt
	python3.9 -m venv ~/sequencer_venv && source ~/sequencer_venv/bin/activate && cd benchmark && pip install -r requirements.txt

bench:
	cd benchmark && export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && fab local

node:
#	./client 127.0.0.1:9006 --size 512 --rate 250 --timeout 1000
	cargo run --bin node --release -- -vvv run --keys ./benchmark/.node-$(N).json --committee ./benchmark/.committee.json --store .db-$(N) --parameters ./benchmark/.parameters.json

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
