.PHONY: init node install-cairo-native run-fibonacci install-cairo-native-linux

init: install-cairo-native
	rustup override set nightly-2023-06-19-aarch64-apple-darwin
	python3.9 -m venv ~/sequencer_venv && source ~/sequencer_venv/bin/activate && cd benchmark && pip install -r requirements.txt

bench:
	cd benchmark && export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && export RUST_LOG=info,salsa=off,cairo_native=off && fab local

N:=0
node: 
	export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && \
	cd benchmark && \
	RUST_LOG=info,salsa=off,cairo_native=off,sled=off cargo run --bin node --features benchmark -- -vvv run --keys ./.node-$(N).json --committee ./.committee.json --parameters ./.parameters.json --store ./.db-$(N)

reset:
	rm -rf ./benchmark/.db-*
	rm -rf ./.db-*

install-cairo-native:
	brew install llvm@16
	brew install tmux
	set -e
	git clone \
		--depth 1 \
		--branch v2.1.0 \
		https://github.com/starkware-libs/cairo.git \
		starkware-cairo
	cp -r starkware-cairo/corelib .
	rm -rf starkware-cairo/

build:
	export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && cargo build --all --release

test:
	@if [ $(shell uname -s) = 'Darwin' ]; then \
		export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16; \
	elif [ $(shell uname -s) = 'Linux' ]; then \
		export MLIR_SYS_160_PREFIX=/usr/lib/llvm-16; \
	fi; \
	echo "+ cargo test"; \
	cargo test 

generate-nodes-keys:
	@if [ -d ./config/committee.json ]; then \
		rm -r ./config/committee.json; \
	fi
	@for node in $(shell seq 0 3); do \
		echo Generating keys for sequencer node $${node}: sequencer/config/sequencer_node$${node}.json; \
		docker compose run sequencer_node0 bash -c "/sequencer/node keys --filename ./sequencer_node$${node}.json && cat ./sequencer_node$${node}.json" > ./config/sequencer_node$${node}.json; \
	done
	@if [ -d ./config/committee.json ]; then \
		rm -r ./config/committee.json; \
	fi

generate-committee: generate-nodes-keys
	@echo "Generating sequencer committee: sequencer/config/committee.json"
	@cd config/scripts && ./generate_committee.py "172.27.0.10, 172.27.0.11, 172.27.0.12, 172.27.0.13"

clippy:
	@if [ $(shell uname -s) = 'Darwin' ]; then \
		export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16; \
	elif [ $(shell uname -s) = 'Linux' ]; then \
		export MLIR_SYS_160_PREFIX=/usr/lib/llvm-16; \
	fi; \
	cargo clippy --all-targets --all-features -- -D warnings
.PHONY: clippy

install-corelibs:
	git clone \
		--depth 1 \
		--branch v2.1.0 \
		https://github.com/starkware-libs/cairo.git \
		starkware-cairo
	cp -r starkware-cairo/corelib .
	rm -rf starkware-cairo/