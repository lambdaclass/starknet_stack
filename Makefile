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

docker-compose-up: clone-madara-explorer docker-build-all
	cd sequencer && make generate-commitee-for-docker
	docker compose up
