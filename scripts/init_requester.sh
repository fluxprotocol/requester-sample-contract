network=${network:-testnet}
accountId=${accountId:-account.testnet}
oracle=${oracle:-09.oracle.flux-dev}
paymentToken=${paymentToken:-v2.wnear.flux-dev}

while [ $# -gt 0 ]; do

   if [[ $1 == *"--"* ]]; then
        param="${1/--/}"
        declare $param="$2"
        # echo $1 $2 // Optional to see the parameter:value result
   fi

  shift
done

NEAR_ENV=$network near call $accountId new "{\"oracle\": \"$oracle\", \"payment_token\": \"$paymentToken\"}" --accountId $accountId