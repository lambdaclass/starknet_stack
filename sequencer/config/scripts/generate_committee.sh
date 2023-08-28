#!/bin/bash

# Define the function to parse IP string
parse_ip_string() {
    IFS=',' read -ra ADDR_ARRAY <<< "$1"
    for ip in "${ADDR_ARRAY[@]}"; do
        ip_list+=("$ip")
    done
}

# Define the function to generate configuration files
generate_config() {
    ip_list=()
    parse_ip_string "$1"
    base_port=9000
    key_files=()
    
    for ((i=0; i<${#ip_list[@]}; i++)); do
        filename="sequencer_node$i.json"
        key_files+=("$filename")
        keys+=("$(cat ../$filename)")
    done
    
    names=("${keys[@]#*\"name\": \"}")
    names=("${names[@]%\"*}")
    
    consensus=()
    front=()
    mempool=()
    
    for ((i=0; i<${#names[@]}; i++)); do
        consensus+=("${ip_list[i]}:$base_port")
        front+=("${ip_list[i]}:$((base_port + ${#names[@]}))")
        mempool+=("${ip_list[i]}:$((base_port + 2 * ${#names[@]}))")
    done
}

if [ $# -ne 1 ]; then
    echo "You need to pass 1 argument: a quoted string with a list of comma-separated IPs"
    exit 1
fi

generate_config "$1"

# Define the JSON output structure
json_output='{
    "consensus": {
        "authorities": {'

# Loop through the generated configuration and add to JSON structure
for ((i=0; i<${#names[@]}; i++)); do
    json_output+="
    \"${names[i]}\""
    json_output+=": {
                \"address\": \"${consensus[i]}\",
                \"name\": \"${names[i]}\",
                \"stake\": 1
            },"
done

json_output+='
        },
        "epoch": 1
    },
    "mempool": {
        "authorities": {'

# Loop through the generated configuration and add to JSON structure
for ((i=0; i<${#names[@]}; i++)); do
    json_output+="
            \"${names[i]}\": {
                \"mempool_address\": \"${mempool[i]}\",
                \"name\": \"${names[i]}\",
                \"stake\": 1,
                \"transactions_address\": \"${front[i]}\"
            },"
done

json_output+='
        },
        "epoch": 1
    }
}'

# Save the JSON output to a file
echo "$json_output" > output.json