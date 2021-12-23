# Requester Sample Contract

Interested in integrating with the Flux Oracle? **Requester Contracts** can be used to create and submit data requests to the Flux Oracle -- here you will find a sample contract to get you started!

On the testnet deployment, anyone can test their own Request Contract with the testnet oracle. When the Oracle is deployed to mainnet, each Requester will require a successful proposal and execution by the Flux DAO. Any protocol or user-deployed smart contract can experiment directly with the Flux Oracle as a data requester and put any kind of data on-chain to be resolved by our pool of testnet validators.

[Please visit the documentation](https://docs.fluxprotocol.org/docs/getting-started/data-requesters) for more information on getting set up as a data requester! Also see [flux-sdk-rs](https://github.com/fluxprotocol/flux-sdk-rs) for types used inside this contract.

## Building and deploying

```bash
# set vars
REQUESTER=requester.account.testnet
ACCOUNT=account.testnet

# create requester account (or use `scripts/reset_account.sh`)
NEAR_ENV=testnet near create-account $REQUESTER --masterAccount $ACCOUNT --initialBalance 5

sh build.sh

bash scripts/deploy_requester.sh --accountId $REQUESTER

# and initialize the account
bash scripts/init_requester.sh --accountId $REQUESTER --oracle 09.oracle.flux-dev --patmentToken v2.wnear.flux-dev

bash scripts/reset_account.sh --master $ACCOUNT --account $REQUESTER
```

## Sending your 1st request

```bash
# set vars
REQUESTER=requester.account.testnet

# send an example arbitrator request at the requester contract that you just deployed
bash ./scripts/new_request.sh --accountId $REQUESTER --paymentToken v2.wnear.flux-dev  --senderId $ACCOUNT

# retrieve the details of the request that you just created (request at index 0)
sh scripts/get_request.sh $REQUESTER 0

# once you've sent more requests, you can call the `get_request` script with the
# exact index number you're interested in to view the details of the request at
# that specific index
```

## Options

### Whitelist

Requesters are encouraged to create mechanisms to ensure domain-specific and high-quality (with definite answers, not spammy, etc.) requests are sent to the Flux Oracle to encourage validators to participate in data resolution.

One option is to whitelist the account(s) allowed to call `create_data_request()` by deploying the contract with an array of account IDs for the `whitelist` parameter in the `init()` method. If left empty, any account will be able to call `create_data_request()`, so another mechanism to limit the number of requests sent to the oracle (e.g. time-based limits, governance controls) is encouraged.
