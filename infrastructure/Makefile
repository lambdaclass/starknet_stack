infrastructure:
	terraform -chdir=terraform/example_sequencer_nodes/ init
	terraform -chdir=terraform/example_sequencer_nodes/ apply
	@echo "Waiting for instances to be ready"
	@sleep 30

ansible-inventory:
	cd ansible/sequencer && \
	python3 generate-inventory.py $(shell aws ec2 describe-instances --region us-west-2 --output text --filters "Name=tag:aws:autoscaling:groupName,Values=starknet_stack_sequencer_0_nodes" "Name=instance-state-name,Values=running" --query 'Reservations[].Instances[].{IP:PublicIpAddress}')

setup-nodes:
	cd ansible/sequencer && \
	ansible-playbook -vv -i inventory.yaml playbooks/sequencer_node.yaml

all: infrastructure ansible-inventory setup-nodes

destroy:
	terraform -chdir=terraform/example_sequencer_nodes/ destroy
