.PHONY: init node install-cairo-native run-fibonacci

init:
	#rustup toolchain install nightly-2023-06-19 --component rustc,cargo,rustfmt
	python3 -m venv ~/sequencer_venv && source ~/sequencer_venv/bin/activate && cd benchmark && pip install -r requirements.txt

bench:
	cd benchmark && export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && fab local

node:
#	./client 127.0.0.1:9006 --size 512 --rate 250 --timeout 1000
	cargo run --bin node --release -- -vvv run --keys ./benchmark/.node-$(N).json --committee ./benchmark/.committee.json --parameters ./benchmark/.parameters.json --store benchmark/.db-0

install-cairo-native:
	git clone https://github.com/lambdaclass/cairo_native.git
#	temporary solution until refactor to new version of cairo_native is done
	cd cairo_native && git checkout 289eddc05e84b99ae37c2889a31abc69f1fedfd1
	brew install llvm@16
	brew install tmux
	cd cairo_native && scripts/fetch-corelibs.sh && export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && cargo +nightly-2023-06-19 build --release

run-fibonacci:
	export MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 && ./cairo_native/target/release/cli run -f fib::fib::main ./cairo_programs/fib.cairo --available-gas 900000000

build-docker:
	docker build -t sequencer:latest .

generate-nodes-keys: init build-docker
	if [ ! -d config/ ]; then \
		mkdir config/; \
	fi
	@for node in $(shell seq 0 3); do \
		echo Generating keys for sequencer node $${node}: config/sequencer_node$${node}.json; \
		docker run sequencer:latest bash -c "/sequencer/node keys --filename ./sequencer_node$${node}.json && cat ./sequencer_node$${node}.json" > ./config/sequencer_node$${node}.json; \
	done

generate-commitee-for-docker: generate-nodes-keys
	cd benchmark && fab config "172.27.0.10, 172.27.0.11, 172.27.0.12, 172.27.0.13"
