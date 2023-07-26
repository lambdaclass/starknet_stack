NODE_COUNT:=6
CLUSTER_NAME:=sequencer_cluster_2
SSH_KEY_NAME:=klaus
INSTANCE_TYPE:=c6a.xlarge
SSH_KEY_DIR:=/Users/klaus/Downloads/klaus_oregon.pem
S3_BUCKET:=terraform-lambdaclass
S3_DIR:=kraken_sequencer
S3_REGION:=us-west-2

infrastructure:
	terraform -chdir=terraform/example_sequencer_nodes/ init -target module.$(CLUSTER_NAME) \
								 -var node_count=$(NODE_COUNT) \
								 -var ssh_key_name=$(SSH_KEY_NAME) \
								 -var instance_type=$(INSTANCE_TYPE) \
								 -var cluster_name=$(CLUSTER_NAME) \
								 -var aws_ami_id=$(AWS_AMI_ID) \
								 -backend-config="bucket=$(S3_BUCKET)" \
								 -backend-config="key=$(S3_DIR)/terraform.tfstate" \
								 -backend-config="region=$(S3_REGION)"
	terraform -chdir=terraform/example_sequencer_nodes/ apply -target module.$(CLUSTER_NAME) \
								  -var node_count=$(NODE_COUNT) \
								  -var ssh_key_name=$(SSH_KEY_NAME) \
								  -var instance_type=$(INSTANCE_TYPE) \
								  -var cluster_name=$(CLUSTER_NAME) \
								  -var aws_ami_id=$(AWS_AMI_ID)
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

# Deploy sequencer on AWS using custom AMI with debian 11 and precompiled sequencer from commit 3e2a28f
sequencer-precompiled: AWS_AMI_ID=ami-0892d553b4b65e371
sequencer-precompiled: infrastructure ansible-inventory nodes-configs setup-nodes-from-ami

# Deploy sequencer on AWS using latest `main` branch commit and compiling it on each node
sequencer: AWS_AMI_ID=ami-0acfc42b227d0b719
sequencer: infrastructure ansible-inventory nodes-configs setup-nodes

destroy-sequencer:
	terraform -chdir=terraform/example_sequencer_nodes/ destroy -target module.$(CLUSTER_NAME) \
								    -var node_count=$(NODE_COUNT) \
								    -var ssh_key_name=$(SSH_KEY_NAME) \
								    -var instance_type=$(INSTANCE_TYPE) \
								    -var cluster_name=$(CLUSTER_NAME) \
								    -var aws_ami_id=$(AWS_AMI_ID)
