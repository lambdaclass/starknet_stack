.PHONY: docker-compose-up

clone-madara-explorer:
	if [ ! -d "madara_explorer" ]; then \
		git clone https://github.com/lambdaclass/madara_explorer.git --branch dockerfile; \
	fi

docker-compose-up: clone-madara-explorer
	cd sequencer && make generate-commitee-for-docker
	docker compose build sequencer_node0
	docker tag starknet_stack_sequencer_node0 starknet_stack_sequencer_node1 && docker tag starknet_stack_sequencer_node0 starknet_stack_sequencer_node2 && docker tag starknet_stack_sequencer_node0 starknet_stack_sequencer_node3
	docker compose up
