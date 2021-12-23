#!/bin/bash

network=${network:-testnet}
accountId=${accountId:-account.testnet}
senderId=${senderId:-sender.testnet}
paymentToken=${paymentToken:-v2.wnear.flux-dev}

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare $param="$2"
        # echo $1 $2 // Optional to see the parameter:value result
   fi

  shift
done

JSON='{\"sources\": [],\"tags\":[\"sports\",\"nfl\"],\"description\":\"Which team won the NFL Super Bowl in 1996?\",\"outcomes\":[\"Cowboys\",\"Steelers\"],\"challenge_period\":\"120000000000\",\"settlement_time\":\"1\",\"data_type\":\"String\",\"creator\":\"your_account_id.flux-dev\"}'
env NEAR_ENV=testnet near call $paymentToken ft_transfer_call "{\"amount\": \"1000000000000000000000000\", \"msg\": \"$JSON\", \"receiver_id\": \"$accountId\"}" --accountId $senderId --amount 0.000000000000000000000001 --gas=300000000000000

# env NEAR_ENV=$network near call $paymentToken ft_transfer_call $ARGS --accountId $accountId --amount 0.000000000000000000000001 --gas=300000000000000