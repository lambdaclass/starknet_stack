.PHONY: docker-compose-up

clone-madara-explorer:
	if [ ! -d "madara_explorer" ]; then \
		git clone https://github.com/lambdaclass/madara_explorer.git --branch dockerfile; \
	fi

docker-build-sequencer:
	docker compose build sequencer_node0
	docker tag starknet_stack_sequencer_node0 starknet_stack_sequencer_node1 && docker tag starknet_stack_sequencer_node0 starknet_stack_sequencer_node2 && docker tag starknet_stack_sequencer_node0 starknet_stack_sequencer_node3

docker-build-watcher:
	docker compose build watcher_prover

docker-build-explorer:
	docker compose build madara_explorer

docker-build-all: docker-build-sequencer docker-build-watcher docker-build-explorer

run-local: clone-madara-explorer docker-build-all
	cd sequencer && make generate-commitee-for-docker
	docker compose up -d
	sleep 15
	docker run --network="starknet_stack_frontend" starknet_stack_sequencer_node0 /sequencer/client 172.27.0.10:9004 --size 256 --rate 250 --timeout 1000 --running-time 10
	docker compose restart madara_explorer
	@echo "Access Madara Explorer in http://localhost:4000/"

stop:
	docker compose down
