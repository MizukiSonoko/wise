# Wise (Who IS ENS)

## How to use

1) Add provider url to environment variables
```sh
export WEB3_PROVIDER=https://mainnet.infura.io/v3/****
```

## How to make contract abi files

```
git clone https://github.com/ensdomains/ens-contracts
cd ens-contracts/
yarn 
yarn build
cat artifacts/contracts/registry/ENS.sol/ENS.json | jq .abi > ENS.abi
cat artifacts/contracts/resolvers/PublicResolver.sol/PublicResolver.json | jq .abi > PublicResolver.abi
mv ENS.abi PublicResolver.abi ../contracts
```
