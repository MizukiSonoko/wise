// Copyright (C) 2021 Mizuki Sonoko. All rights reserved.

use web3::{
    contract::{Contract, Options},
    types::{Address, Bytes, H256},
    Transport,
};

pub struct ResolverContract<T: Transport> {
    contract: Contract<T>,
}

impl<T: Transport> ResolverContract<T> {
    pub fn new(addr: Address, web3: web3::Web3<T>) -> Self {
        ResolverContract {
            contract: Contract::from_json(
                web3.eth(),
                addr,
                include_bytes!("../contracts/PublicResolver.abi"),
            )
            .unwrap(),
        }
    }

    pub async fn content_hash(&self, namehash: &H256) -> Result<Bytes, web3::contract::Error> {
        let res = self
            .contract
            .query("contenthash", *namehash, None, Options::default(), None);
        let r = res.await.expect("contenthash.result.wait()");
        Ok(r)
    }

    pub async fn text(
        &self,
        namehash: &H256,
        name: String,
    ) -> Result<String, web3::contract::Error> {
        let res = self
            .contract
            .query("text", (*namehash, name), None, Options::default(), None);
        let r = res.await.expect("text.result.wait()");
        Ok(r)
    }
}
