infrastructure:
	@terraform -chdir=infrastructure/terraform/example_sequencer_nodes/ init
	@terraform -chdir=infrastructure/terraform/example_sequencer_nodes/ apply

ansible-inventory:
	cd infrastructure/ansible/; \
	python3 generate-inventory.py $(shell aws ec2 describe-instances --region us-west-2 --output text --filters "Name=tag:aws:autoscaling:groupName,Values=starknet_stack_sequencer_0_nodes" "Name=instance-state-name,Values=running" --query 'Reservations[].Instances[].{IP:PublicIpAddress}')

setup-nodes:
	cd infrastructure/ansible/; \
	ansible-playbook -i inventory.yaml playbooks/sequencer_node.yaml

all: infrastructure ansible-inventory setup-nodes
