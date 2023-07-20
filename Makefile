.PHONY: docker-compose-up

clone-madara-explorer:
	if [ ! -d "madara_explorer" ]; then \
		git clone https://github.com/lambdaclass/madara_explorer.git --branch main; \
	fi

docker-build-sequencer:
	docker compose build sequencer_node0

docker-build-watcher:
	docker compose build watcher_prover

docker-build-explorer:
	docker compose build madara_explorer

docker-build-all: docker-build-sequencer docker-build-watcher docker-build-explorer

run-local: clone-madara-explorer docker-build-all
	cd sequencer && make generate-commitee-for-docker
	docker compose up -d
	@echo "Populating sequencer with sample transactions..."
	@sleep 20
	@echo Restarting Madara Explorer
	docker compose restart madara_explorer
	@sleep 5
	@echo "Access Madara Explorer in http://localhost:4000/"

stop:
	docker compose down
