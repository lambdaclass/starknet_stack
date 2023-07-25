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
		--branch v2.0.1 \
		https://github.com/starkware-libs/cairo.git \
		starkware-cairo
	cp -r starkware-cairo/corelib .
	rm -rf starkware-cairo/

build:
	export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && cargo build --all --release

test:
	export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && cargo test

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

generate-committee-for-docker: generate-nodes-keys
	@echo "Generating sequencer committee: sequencer/config/committee.json"
	@cd config/scripts && ./generate_committee.py "172.27.0.10, 172.27.0.11, 172.27.0.12, 172.27.0.13"

clippy:
	cargo clippy --all-targets --all-features -- -D warnings
.PHONY: clippy

install-cairo-native-linux:
	sudo echo "deb http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-16 main" > /etc/apt/sources.list.d/llvm-16.list
	sudo echo "deb-src http://apt.llvm.org/bullseye/ llvm-toolchain-bullseye-16 main" >> /etc/apt/sources.list.d/llvm-16.list
	wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
	apt update -y
	apt install -y llvm-16 \
				libmlir-16-dev \
				mlir-16-tools \
				libpolly-16-dev
