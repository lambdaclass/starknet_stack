.PHONY: docker-compose-up

clone-madara-explorer:
	if [ ! -d "madara_explorer" ]; then \
		git clone https://github.com/lambdaclass/madara_explorer.git --branch dockerfile; \
	fi

docker-compose-up: clone-madara-explorer
	cd sequencer && make generate-commitee-for-docker
	docker compose up
