use anyhow::Result;
use futures::future::join_all;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::telnet::client::Client;
use crate::telnet::shell;
use crate::{config, eprty, prty};

// Setup TelnetOptions
async fn nigotate(conn: &mut TcpStream) -> Result<()> {
    conn.write_all(&[0xFF, 0xFB, 0x01, 0xFF, 0xFB, 0x03, 0xFF, 0xFC, 0x22])
        .await?;
    // See How Much Data Is Comming in and read it off
    let mut buf = [0 as u8; 1200];
    conn.read(&mut buf).await?;

    Ok(())
}

async fn serve(s: TcpListener, bin: String, exec_args: Vec<String>) {
    while let Ok((mut conn, addr)) = s.accept().await {
        let bin = bin.clone();
        let exec_args = exec_args.clone();
        tokio::spawn(async move {
            prty!("new connection, addr: {}", addr.ip());

            if let Err(why) = nigotate(&mut conn).await {
                eprty!("{} failed to nigotiate, due to error {}", addr.ip(), why)
            }

            // Now That Telnet Has Nigotiatd We Can Start
            // Build the Client Data And Send It Off To A systemd-nspawn Shell
            let mut c = Client::new(conn, addr.ip()).unwrap();
            let _ = shell::emulator(&mut c, bin, exec_args).await;
        });
    }
}

pub async fn telnetstart(c: config::data::Data) {
    // Ongoing Tcp Sockets Waiting For a Connection
    let mut threads = vec![];

    for port in c.telnetports {
        match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
            Ok(stream) => {
                let exec_binclone = c.bin.clone();
                let exec_argsclone = c.args.clone();
                
                prty!("port: {}, started!", port);
                threads.push(tokio::spawn(async {
                    serve(stream, exec_binclone, exec_argsclone).await
                }));
            }
            Err(why) => eprty!("failed to start telnet port: {}", why),
        }
    }

    join_all(threads).await;
}
