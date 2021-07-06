// Copyright (C) 2021 Mizuki Sonoko. All rights reserved.

use clap::{App, Arg};
extern crate hex;
extern crate web3;

mod ens;
mod resolver;

use ens::EnsContract;
use resolver::ResolverContract;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::{env, process};
use std::{io, io::prelude::*};
use thiserror::Error;
use tiny_keccak::{Hasher, Keccak};
use web3::types::H256;

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

fn is_name(val: &str) -> Result<()> {
    if val.ends_with(".eth") {
        Ok(())
    } else {
        Err(WiseError::InvalidArgvName("the name format must be ***.eth".to_string()).into())
    }
}

fn strip(val: &str) -> String {
    val[1..val.len() - 1].to_string()
}

#[derive(Error, Debug)]
pub enum WiseError {
    #[error("decode failed")]
    DecodeFailed(hex::FromHexError),
    #[error("decode failed content hash to utf8")]
    InvalidContentHash(std::string::FromUtf8Error),
    #[error("the name format must be ***.eth")]
    InvalidArgvName(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Supported type is ipfs, swarm, ipns.
// Referring source https://github.com/pldespaigne/content-hash/blob/master/src/profiles.js
fn decode_content_hash(val: &str) -> Result<(String, String)> {
    if val.starts_with("0xe3") {
        // ipfs-ns
        // Note: ens's content hash is
        //    0xe301017012205cf128dcc4ef93cb5b900d30540ce1ab25328e450c7f5f9b3a6d338a2f8c1294
        // IPFS hash is 12205cf128dcc4ef93cb5b900d30540ce1ab25328e450c7f5f9b3a6d338a2f8c1294
        // So remove prefix 0xe3010170
        let vec = hex::decode(&val[10..val.len()])?;
        return Ok((bs58::encode(&vec).into_string(), "ipfs-ns".to_string()));
    } else if val.starts_with("0xe4") {
        // swarm-ns
        Ok((
            String::from_utf8(hex::decode(&val[2..val.len()])?)?,
            "swarm-ns".to_string(),
        ))
    } else if val.starts_with("0xe5") {
        // ipns-ns
        let vec = hex::decode(&val[11..val.len() - 1])?;
        return Ok((bs58::encode(&vec).into_string(), "ipns-ns".to_string()));
    } else {
        Ok((
            String::from_utf8(hex::decode(&val[2..val.len()])?)?,
            "utf-8".to_string(),
        ))
    }
}

#[derive(Serialize, Deserialize)]
struct EnsInfo {
    owner: web3::types::H160,
    resolver: web3::types::H160,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decoded_content_hash: Option<String>,
    text_record: HashMap<String, String>,
}

async fn fetch_info(
    transport: &web3::transports::Http,
    ens_name: &str,
    is_json: &bool,
    is_qr: &bool,
    is_only_addr: &bool,
) -> Result<()> {
    let ens_namehash: H256 = H256::from_slice(namehash(ens_name).as_slice());
    let ens = EnsContract::new(web3::Web3::new(&transport));
    let mut ens_info = EnsInfo {
        owner: web3::types::H160([0u8; 20]),
        resolver: web3::types::H160([0u8; 20]),
        content_hash: None,
        content_type: None,
        decoded_content_hash: None,
        text_record: HashMap::new(),
    };

    match ens.owner(&ens_namehash).await {
        Ok(owner) => {
            if *is_only_addr {
                if owner == web3::types::H160([0u8; 20]) {
                    println!("  {:12} : avaliable", &ens_name[0..ens_name.len() - 4]);
                } else {
                    println!("  {:12} : {:?}", &ens_name[0..ens_name.len() - 4], owner);
                }
                return Ok(());
            }

            if owner == web3::types::H160([0u8; 20]) {
                if *is_json {
                    println!(
                        "{}",
                        json!({ "result": format!("owner not found name: {}", ens_name) })
                            .to_string()
                    );
                    process::exit(0);
                }
                println!("owner not found name: {}\n", ens_name);
                println!(
                    "you can register at https://app.ens.domains/name/{}\n",
                    ens_name
                );
                process::exit(0);
            }
            ens_info.owner = owner;
        }
        Err(err) => {
            println!("err: {}", err);
            process::exit(1);
        }
    }
    match ens.resolver(&ens_namehash).await {
        Ok(rslv_addr) => {
            if rslv_addr != web3::types::H160([0u8; 20]) {
                ens_info.resolver = rslv_addr;
                let resolver = ResolverContract::new(rslv_addr, web3::Web3::new(&transport));
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
                    "snapshot",
                    "com.github",
                ];
                for param in params.iter() {
                    match resolver.text(&ens_namehash, param.to_string()).await {
                        Ok(value) => {
                            if !value.is_empty() {
                                ens_info.text_record.insert(param.to_string(), value);
                            }
                        }
                        Err(err) => {
                            println!("\terr: {}", err)
                        }
                    }
                }

                match resolver.content_hash(&ens_namehash).await {
                    Ok(bytes_content_hash) => {
                        let content_hash = strip(
                            &String::from_utf8(serde_json::to_vec(&bytes_content_hash).unwrap())
                                .unwrap(),
                        );
                        if content_hash != "0x" {
                            ens_info.content_hash = Some(content_hash);
                            if ens_info.content_hash.is_some() {
                                let decoded =
                                    decode_content_hash(ens_info.content_hash.as_ref().unwrap())
                                        .expect("decode failed");
                                ens_info.decoded_content_hash = Some(decoded.0);
                                ens_info.content_type = Some(decoded.1);
                            }
                        }
                    }
                    Err(err) => {
                        println!("\terr: {}", err)
                    }
                }
            }
        }
        Err(err) => {
            println!("\terr: {}", err);
            process::exit(1);
        }
    }

    if *is_json {
        println!("{}", json!(ens_info).to_string());
    } else if *is_qr {
        qr2term::print_qr(format!("{:?}", ens_info.owner)).unwrap();
        println!(
            "----\nens:{} owner_address {:?}\n",
            ens_name, ens_info.owner,
        );
    } else {
        println!("\n       owner: {:?}", ens_info.owner);
        println!("    resolver: {:?}", ens_info.resolver);
        println!("\n-------\n");
        if ens_info.content_hash.is_some() {
            println!(" content_hash : {}", &ens_info.content_hash.unwrap());
            println!(
                " decoded_hash : {}",
                &ens_info.decoded_content_hash.unwrap()
            );
            println!(" content_type : {}", &ens_info.content_type.unwrap());
        } else {
            println!("content_hash: Not set");
        }
        println!("\n-------\n");
        for (name, value) in &ens_info.text_record {
            println!(" {:<12}: {}", name, value);
        }
        println!()
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let provider_addr = match env::var("WEB3_PROVIDER") {
        Ok(val) => val,
        Err(err) => {
            println!("env 'WEB3_PROVIDER' is not setted, {}", err);
            process::exit(1);
        }
    };
    let transport = web3::transports::Http::new(&provider_addr)?;

    let matches = App::new("wise")
        .about("cli tool for ENS")
        .arg(
            Arg::new("name")
                .about("Searching name like (mizuki.eth)")
                .index(1)
                .validator(is_name),
        )
        .arg(
            Arg::new("json")
                .short('j')
                .long("json")
                .about("Result will be json format"),
        )
        .arg(
            Arg::new("qr")
                .short('q')
                .long("qr")
                .about("Print QR code of Address"),
        )
        .get_matches();

    // Option
    let is_json = matches.is_present("json");
    let is_qr = matches.is_present("qr");

    let ens_name = matches.value_of("name");
    if ens_name.is_some() {
        return fetch_info(&transport, ens_name.unwrap(), &is_json, &is_qr, &false).await;
    } else {
    }

    // Check stdin input
    // Note: I could not solve 'The following required arguments were not provided'

    let mut names = String::new();
    for l in io::stdin().lock().lines().map(|ln| ln.unwrap()) {
        names += &l;
    }

    if names.len() == 0 {
        println!("The following required arguments were not provided:");
        println!("\t<name>: like mizuki.eth");
        println!("USAGE:");
        println!("wise [FLAGS] <name>\n");
        println!("For more information try --help");
    } else {
        for name in names.split_whitespace() {
            fetch_info(
                &transport,
                &(name
                    .replace(&['(', ')', ',', '\"', '.', ';', ':', '\''][..], "")
                    .to_string()
                    + &".eth".to_string()),
                &false,
                &false,
                &true,
            )
            .await
            .expect("fetch err")
        }
    }
    Ok(())
}
