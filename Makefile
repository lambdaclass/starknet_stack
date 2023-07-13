.PHONY: docker-compose-up

clone-madara-explorer:
	git clone https://github.com/lambdaclass/madara_explorer.git --branch dockerfile

docker-compose-up: clone-madara-explorer
	cd sequencer && make generate-commitee-for-docker
	docker compose up
