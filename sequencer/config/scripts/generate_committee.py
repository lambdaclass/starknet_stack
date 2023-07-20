#!/usr/bin/env python3

import subprocess, sys

from config import RemoteCommittee, Key

def generate_config(ips):
    ''' '''
    # Generate configuration files.
    keys = []
    ips = [ip.strip() for ip in ips.split(',')]
    base_port = 9000
    key_files = [f'sequencer_node{i}.json' for i in range(len(ips))]
    for filename in key_files:
        keys += [Key.from_file(f'../{filename}')]

    names = [x.name for x in keys]
    committee = RemoteCommittee(names, base_port, ips)
    committee.print('../committee.json')

if len(sys.argv) != 2:
  print("You need to pass 1 argument, a quoted string with the list of comma-separated ips")
  sys.exit(1)

generate_config(sys.argv[1])
