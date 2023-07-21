#!/usr/bin/env python

from yaml import dump
from sys import argv

ip_list = argv[1:]

node = {
    "ansible_host": "{IP}",
    "ansible_user": "admin",
    "ansible_python_interpreter": "/usr/bin/python3",
    "ansible_ssh_private_key_file": "{ANSIBLE_SSH_PKEY}",
    "ansible_ssh_extra_args": "-o StrictHostKeyChecking=no"
}

obj = {"nodes": {"hosts": {}}}

for i in range(0, len(ip_list)):
    obj["nodes"]["hosts"]["node%s" % i] = dict()
    for key in node:
        obj["nodes"]["hosts"]["node%s" % i][key] = node[key].format(IP = ip_list[i], ANSIBLE_SSH_PKEY = "{{ lookup(\"env\",\"ANSIBLE_SSH_PKEY\") }}")

with open("inventory.yaml", "w") as file:
    file.write(dump(obj))
