.PHONY: docker-compose-up

clone-madara-explorer:
	if [ ! -d "madara_explorer" ]; then \
		git clone --recurse-submodules https://github.com/lambdaclass/madara_explorer.git --branch starknet-stack-explorer && git submodule update --init --recursive; \
	fi

docker-build-sequencer:
	docker compose build sequencer_node0

docker-build-watcher:
	docker compose build watcher_prover

docker-build-explorer:
	docker compose build madara_explorer

docker-build-all: docker-build-sequencer docker-build-watcher docker-build-explorer

run-local: clone-madara-explorer docker-build-all
	cd sequencer && make generate-committee
	docker compose up -d
	@echo "Populating sequencer with sample transactions..."
	docker compose logs -f sequencer_client0
	@echo Restarting Madara Explorer
	docker compose restart madara_explorer
	@sleep 5
	@echo "Access Madara Explorer in http://localhost:4000/"

run-client:
	docker run --network="starknet_stack_frontend" starknet_stack-sequencer_node0 /sequencer/client 172.27.0.10:9004 --size 256 --rate 250 --timeout 1000 --running-time 10

client-remote:
	ifndef MEMPOOL_IP
	$(error MEMPOOL_IP is not set)
	endif
	cd sequencer && cargo run --bin client --release --features benchmark --  $(MEMPOOL_IP):9004 --size 256 --rate 100 --timeout 1000 --running-time 15

stop:
	docker compose down
