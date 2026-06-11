use std::{fs::File, io::Read};

use anyhow::Result;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Data {
    pub telnetports: Vec<u16>,
    pub sshports: Vec<u16>,
    pub bin: String,
    pub args: Vec<String>,
}

const CONFIGFILEPATH: &str = "./config.json";

pub fn get() -> Result<Data> {
    let mut buf = String::new();
    File::open(CONFIGFILEPATH)?.read_to_string(&mut buf)?;
    Ok(serde_json::from_str(&buf)?)
}
