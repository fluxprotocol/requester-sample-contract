#!/bin/bash

[[ -z "$1" ]] && echo "Expected syntax: $0 YOUR_TESTNET_ACCOUNT_ID YOUR_REQUEST_NONCE" >&2 && exit 1
[[ -z "$2" ]] && echo "Expected syntax: $0 YOUR_TESTNET_ACCOUNT_ID YOUR_REQUEST_NONCE" >&2 && exit 1

near call $1 get_data_request "{\"nonce\": \"$2\"}" --accountId $1
near call $1 get_data_response "{\"nonce\": \"$2\"}" --accountId $1