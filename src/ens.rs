// Copyright (C) 2021 Mizuki Sonoko. All rights reserved.

use web3::{
    contract::{Contract, Options},
    types::{Address, H256},
    Transport,
};

// See: https://etherscan.io/address/0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e
// Doc: https://docs.ens.domains/ens-deployments
const ENS_CONTRACT_ADDR: &str = "00000000000C2E074eC69A0dFb2997BA6C7d2e1e";

pub struct EnsContract<T: Transport> {
    contract: Contract<T>,
}

impl<T: Transport> EnsContract<T> {
    pub fn new(web3: web3::Web3<T>) -> Self {
        EnsContract {
            contract: Contract::from_json(
                web3.eth(),
                ENS_CONTRACT_ADDR.parse().unwrap(),
                include_bytes!("../contracts/ENS.abi"),
            )
            .unwrap(),
        }
    }

    pub async fn owner(&self, namehash: &H256) -> Result<Address, web3::contract::Error> {
        self.contract
            .query("owner", (*namehash,), None, Options::default(), None)
            .await
    }

    pub async fn resolver(&self, namehash: &H256) -> Result<Address, web3::contract::Error> {
        self.contract
            .query("resolver", (*namehash,), None, Options::default(), None)
            .await
    }
}
