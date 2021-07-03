// Copyright (C) 2021 Mizuki Sonoko. All rights reserved.

use clap::{App, Arg};
extern crate web3;

use std::{env, process};
use tiny_keccak::{Hasher, Keccak};
use web3::{
    contract::{Contract, Options},
    types::{Address, H256},
    Transport,
};

pub fn namehash(name: &str) -> Vec<u8> {
    let mut node = vec![0u8; 32];
    if name.is_empty() {
        return node;
    }
    let mut labels: Vec<&str> = name.split('.').collect();
    labels.reverse();
    for label in labels.iter() {
        let mut labelhash = [0u8; 32];
        let mut kekkak1 = Keccak::v256();
        kekkak1.update(label.as_bytes());
        kekkak1.finalize(&mut labelhash);
        node.append(&mut labelhash.to_vec());

        labelhash = [0u8; 32];
        let mut kekkak2 = Keccak::v256();
        kekkak2.update(node.as_slice());
        kekkak2.finalize(&mut labelhash);
        node = labelhash.to_vec();
    }
    node
}

fn is_name(val: &str) -> Result<(), String> {
    if val.ends_with(".eth") {
        Ok(())
    } else {
        Err(String::from("the name format must be ***.eth"))
    }
}

// See: https://etherscan.io/address/0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e
// Doc: https://docs.ens.domains/ens-deployments
const ENS_CONTRACT_ADDR: &str = "00000000000C2E074eC69A0dFb2997BA6C7d2e1e";

struct EnsContract<T: Transport> {
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

struct ResolverContract<T: Transport> {
    contract: Contract<T>,
}

impl<T: Transport> ResolverContract<T> {
    pub fn new(addr: web3::types::Address, web3: web3::Web3<T>) -> Self {
        ResolverContract {
            contract: Contract::from_json(
                web3.eth(),
                addr,
                include_bytes!("../contracts/PublicResolver.abi"),
            )
            .unwrap(),
        }
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

    pub async fn print_text(&self, namehash: &H256, name: String) {
        match self.text(&namehash, name.to_string()).await {
            Ok(value) => {
                if !value.is_empty() {
                    println!(" {:<12}: {}", name, value)
                } else {
                    println!(" {:<12}: Not set", name)
                }
            }
            Err(err) => {
                println!("\terr: {}", err)
            }
        }
    }
}

#[tokio::main]
async fn main() -> web3::Result<()> {
    let matches = App::new("wise")
        .arg(
            Arg::new("name")
                .about("searching name like (mizuki.eth)")
                .index(1)
                .required(true)
                .validator(is_name),
        )
        .arg(
            Arg::new("json")
                .short('j')
                .about("Result will be json format"),
        )
        .get_matches();

    if matches.is_present("json") {
        println!("json is not supported");
    }
    let ens_name = matches.value_of("name").unwrap();
    let ens_namehash: H256 = H256::from_slice(namehash(ens_name).as_slice());
    let provider_addr = match env::var("WEB3_PROVIDER") {
        Ok(val) => val,
        Err(err) => {
            println!("env 'WEB3_PROVIDER' is not setted, {}", err);
            process::exit(1);
        }
    };
    let transport = web3::transports::Http::new(&provider_addr)?;
    let ens = EnsContract::new(web3::Web3::new(&transport));

    match ens.owner(&ens_namehash).await {
        Ok(owner) => {
            if owner == web3::types::H160([0u8; 20]) {
                println!("owner not found name:{}\n", ens_name);
                process::exit(0);
            }
            println!("\nowner: {:?}", owner);
        }
        Err(err) => {
            println!("err: {}", err)
        }
    }

    match ens.resolver(&ens_namehash).await {
        Ok(rslv_addr) => {
            println!("resolver: {:?}", rslv_addr);
            let resolver = ResolverContract::new(rslv_addr, web3::Web3::new(&transport));
            println!("\n-------\n");
            let params = vec![
                "vnd.twitter",
                "vnd.github",
                "url",
                "email",
                "avatar",
                "description",
                "notice",
                "keywords",
                "com.twitter",
                "com.github",
            ];
            for param in params.iter() {
                resolver.print_text(&ens_namehash, param.to_string()).await;
            }
        }
        Err(err) => {
            println!("\terr: {}", err)
        }
    }

    Ok(())
}
