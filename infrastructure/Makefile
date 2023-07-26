NODE_COUNT:=6
CLUSTER_NAME:=sequencer_cluster_2
SSH_KEY_NAME:=klaus
INSTANCE_TYPE:=c6a.xlarge
AWS_AMI_ID:=ami-0892d553b4b65e371
SSH_KEY_DIR=/Users/klaus/Downloads/klaus_oregon.pem
infrastructure:
	terraform -chdir=terraform/example_sequencer_nodes/ init -target module.$(CLUSTER_NAME) \
								 -var node_count=$(NODE_COUNT) \
								 -var ssh_key_name=$(SSH_KEY_NAME) \
								 -var instance_type=$(INSTANCE_TYPE) \
								 -var cluster_name=$(CLUSTER_NAME)
	terraform -chdir=terraform/example_sequencer_nodes/ apply -target module.$(CLUSTER_NAME) \
								  -var node_count=$(NODE_COUNT) \
								  -var ssh_key_name=$(SSH_KEY_NAME) \
								  -var instance_type=$(INSTANCE_TYPE) \
								  -var cluster_name=$(CLUSTER_NAME)
	@echo "Waiting for instances to be ready"
	@sleep 30

ansible-inventory:
	cd ansible/sequencer && \
	python3 generate-inventory.py $(shell aws ec2 describe-instances --region us-west-2 --output text --filters "Name=tag:aws:autoscaling:groupName,Values=$(CLUSTER_NAME)_nodes" "Name=instance-state-name,Values=running" --query 'Reservations[].Instances[].{IP:PublicIpAddress}')

nodes-configs:
	@cd ../sequencer && \
	echo "Generating sequencer nodes keys and committee file" && \
	if [ -d ./config/committee.json ]; then \
		rm -r ./config/committee.json; \
	fi; \
	for node in $(shell seq 0 $(NODE_COUNT)); do \
		echo Generating keys for sequencer node $${node}: sequencer/config/sequencer_node$${node}.json; \
		docker run starknet_stack-sequencer_node0 bash -c "/sequencer/node keys --filename ./sequencer_node$${node}.json && cat ./sequencer_node$${node}.json" > ./config/sequencer_node$${node}.json; \
	done; \
	if [ -d ./config/committee.json ]; then \
		rm -r ./config/committee.json; \
	fi; \
	cd config/scripts && ./generate_committee.py "$(shell aws ec2 describe-instances --region us-west-2 --output text --filters "Name=tag:aws:autoscaling:groupName,Values=$(CLUSTER_NAME)_nodes" "Name=instance-state-name,Values=running" --query 'Reservations[].Instances[].{IP:PrivateIpAddress}')"
	@cp ../sequencer/config/committee.json ansible/sequencer/playbooks/files/
	@for node_json in $(shell ls ../sequencer/config/sequencer_node*); do \
		cp $${node_json} ansible/sequencer/playbooks/files/; \
	done

setup-nodes:
	cd ansible/sequencer && \
	ansible-playbook -v -i inventory.yaml playbooks/sequencer_node.yaml

setup-nodes-from-ami:
	cd ansible/sequencer && \
	export ANSIBLE_SSH_PKEY=$(SSH_KEY_DIR) && \
	ansible-playbook -v -i inventory.yaml playbooks/from_ami_sequencer_node.yaml

sequencer-precompiled: infrastructure ansible-inventory nodes-configs setup-nodes-from-ami

sequencer: infrastructure ansible-inventory nodes-configs setup-nodes

destroy:
	terraform -chdir=terraform/example_sequencer_nodes/ destroy -target module.$(CLUSTER_NAME) \
								    -var node_count=$(NODE_COUNT) \
								    -var ssh_key_name=$(SSH_KEY_NAME) \
								    -var instance_type=$(INSTANCE_TYPE) \
								    -var cluster_name=$(CLUSTER_NAME)
