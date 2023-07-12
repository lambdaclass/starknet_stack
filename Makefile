.PHONY: docker-compose-up

docker-compose-up:
	cd sequencer && make generate-commitee-for-docker
	docker compose up
