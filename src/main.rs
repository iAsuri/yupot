use crate::telnet::server::telnetstart;

pub mod prtylog;
pub mod telnet;
pub mod config;

#[tokio::main]
async fn main() {
    prty!("Parsing Config");
    let config = match config::data::get() {
        Ok(c) => {
            largeprty!("{:#?}", c);
            prty!("Config Parsed!, Starting Server");
            c
        },
        Err(why) => {
            eprty!("{:?}", why);
            return;
        },
    };


    telnetstart(config).await;
}
