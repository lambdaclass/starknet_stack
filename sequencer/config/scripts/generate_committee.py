#!/usr/bin/env python3

import sys, ipaddress

from config import RemoteCommittee, Key

def parse_ip_string(ip_string):
    if ',' in ip_string:
        ip_list = [ip.strip() for ip in ip_string.split(',')]
    else:
        ip_list = [ip.strip() for ip in ip_string.split(' ')]
    for ip in ip_list:
        try:
            ipaddress.ip_address(ip)
        except ValueError:
            print(f'{ip} is not a valid IP address')
            sys.exit(1)
    return ip_list

def generate_config(ips):
    ''' '''
    # Generate configuration files.
    keys = []
    ip_list = parse_ip_string(ips)
    base_port = 9000
    key_files = [f'sequencer_node{i}.json' for i in range(len(ip_list))]
    for filename in key_files:
        keys += [Key.from_file(f'../{filename}')]

    names = [x.name for x in keys]
    committee = RemoteCommittee(names, base_port, ip_list)
    committee.print('../committee.json')

if len(sys.argv) != 2:
    print("You need to pass 1 argument: a quoted string with a list of comma-separated IPs")
    sys.exit(1)


class RemoteCommittee(Committee):
    def __init__(self, names, port, nodes_ips):
        assert isinstance(names, list) and all(
            isinstance(x, str) for x in names)
        assert isinstance(port, int)
        size = len(names)
        consensus = [f'{nodes_ips[i]}:{port}' for i in range(size)]
        front = [f'{nodes_ips[i]}:{port + size}' for i in range(size)]
        mempool = [f'{nodes_ips[i]}:{port + 2*size}' for i in range(size)]
        super().__init__(names, consensus, front, mempool)

generate_config(sys.argv[1])
