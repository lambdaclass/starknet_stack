.PHONY: docker-compose-up

clone-madara-explorer:
	if [ ! -d "madara_explorer" ]; then \
		git clone https://github.com/lambdaclass/madara_explorer.git --branch main; \
	fi

docker-build-sequencer:
	docker compose build sequencer_node0
	docker tag starknet_stack-sequencer_node0 starknet_stack-sequencer_node1 && docker tag starknet_stack-sequencer_node0 starknet_stack-sequencer_node2 && docker tag starknet_stack-sequencer_node0 starknet_stack-sequencer_node3

docker-build-watcher:
	docker compose build watcher_prover

docker-build-explorer:
	docker compose build madara_explorer

docker-build-all: docker-build-sequencer docker-build-watcher docker-build-explorer

run-local: clone-madara-explorer docker-build-all
	cd sequencer && make generate-commitee-for-docker
	docker compose up -d
	@sleep 15
	@echo "Populating sequencer with sample transactions..."
	docker run --network="starknet_stack_frontend" starknet_stack-sequencer_node0 /sequencer/client 172.27.0.10:9004 --size 256 --rate 250 --timeout 1000 --running-time 10
	@echo Restarting Madara Explorer
	docker compose restart madara_explorer
	@sleep 5
	@echo "Access Madara Explorer in http://localhost:4000/"

stop:
	docker compose down
