# Wise (Who IS ENS)
Whois command for ENS.

https://user-images.githubusercontent.com/6281583/124363506-917be680-dc76-11eb-8e14-7e4160be83ea.mov

```
$ wise mizuki.eth

       owner: 0xdb10e4a083b87e803594c12c679422dce5fcccb9
    resolver: 0x4976fb03c32e5b8cfe2b6ccb31c09ba78ebaba41

-------

content_hash: Not set

-------

 vnd.twitter : mizuki_sonoko
 vnd.github  : MizukiSonoko
 url         : https://mizuki.io

```

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
