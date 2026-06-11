use std::{
    fs::OpenOptions,
    io::Write,
};

use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::TcpStream,
};

use crate::{eprty, largeprty};

pub struct Client {
    pub conn: TcpStream,
    pub ip: String,

    // Logging Data
    pub bash_log: String,
    pub login_history: Vec<(String, String)>,
    pub commands_written: u32,
}

pub fn log_data(path: &str, data: &[u8]) -> Result<()> {
    let path = std::path::Path::new(path);
    let prefix = path.parent().unwrap();
    let _ = std::fs::create_dir_all(prefix);

    let mut fs = OpenOptions::new().create(true).append(true).open(path)?;

    fs.write(data)?;

    return Ok(());
}

impl Drop for Client {
    fn drop(&mut self) {
        largeprty!(
            "Host: {}, closed.\n\t - creds used: [{:?}]\n\t - newlines (aka commands): [{}]",
            self.ip,
            self.login_history,
            self.commands_written
        );

        // commands Logged
        if let Err(why) = log_data(
            &format!("./logs/{}/commands", self.ip),
            self.bash_log.as_bytes(),
        ) {
            eprty!("[{}] failed to log, due to error: [{}]", self.ip, why);
        }

        if let Err(why) = log_data(
            &format!("./logs/{}/login", self.ip),
            format!("{:?}\r\n", self.login_history).as_bytes(),
        ) {
            eprty!("[{}] failed to log, due to error: [{}]", self.ip, why);
        }
    }
}

impl Client {
    pub fn new(
        conn: TcpStream,
        addr: std::net::IpAddr,
    ) -> anyhow::Result<Self> {
        Ok(Client {
            conn,
            ip: addr.to_string(),

            bash_log: String::new(),
            login_history: vec![],
            commands_written: 0,
        })
    }

    pub async fn write(&mut self, text: &str) -> Result<usize> {
        Ok(self.conn.write(text.as_bytes()).await?)
    }

    // Writer Wrapper For Pty Session

    // regulat writer wrapper for socket
    #[inline]
    pub async fn regwrite(&mut self, buf: &[u8]) -> Result<usize> {
        self.bash_log.push(buf[0].into());
        Ok(self.conn.write(buf).await?)
    }

    #[inline]
    // Checks If Socket is ready to be read
    pub async fn client_is_ready(&mut self) -> Result<bool> {
        Ok(self.conn.ready(Interest::READABLE).await?.is_readable())
    }

    pub async fn read(&mut self) -> Result<String> {
        let mut ret_str = String::new();

        loop {
            let mut buf = [0; 1];
            self.conn.read(&mut buf).await?;

            match buf[0] {
                127 => {
                    ret_str.pop();
                    self.conn.write(&buf).await?;
                }

                b'\n' => {
                    self.write("\r\n").await?;
                    break;
                }
                b'\r' => continue,
                // Ascii Printable characters
                32..=126 => {
                    ret_str.push(buf[0].into());
                    self.conn.write(&buf).await?;
                }
                _ => continue,
            }
        }

        Ok(ret_str)
    }
}
