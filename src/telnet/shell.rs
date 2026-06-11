use nix::libc::size_t;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
};

use crate::{
    largeprty,
    telnet::{client::Client, pty},
};

const PTYBUFFER: size_t = 1200;

pub async fn emulator(c: &mut Client, bin: String, args: Vec<String>) -> anyhow::Result<()> {
    // Username And Password What Ever Then Continue On
    loop {
        c.write("login: ").await?;
        let username = c.read().await?;

        c.write("Password: ").await?;
        let password = c.read().await?;

        // Push Data To Logger
        c.login_history.push((username.clone(), password.clone()));

        largeprty!(
            "client login: {}\n\t - username: {:?}\n\t - password: {:?}",
            c.ip,
            username,
            password
        );

        if username == "root" && password == "root" {
            break;
        }
    }

    let pty = pty::Asyncpty::new(bin.into(), args)?;

    loop {
        let mut clientbuf = [0u8; 1];
        let mut pty_buf = [0u8; PTYBUFFER];

        select! {
                    // Read From Temrinal buffer and write to client
            val = pty.read(&mut pty_buf, PTYBUFFER-1) => {
                if let Some(count) = val? {
                    c.bash_log.push_str(
                        &String::from_utf8_lossy(&pty_buf[..count as usize])
                    );
                    c.conn.write(&pty_buf[..count as usize]).await?;
                }
            }
                    // Read From Client write to terminal
            val = c.conn.read(&mut clientbuf) => {
           let byte = clientbuf[0];
            match byte {
                b'\r' => continue,
                b'\n' => c.commands_written += 1,
                _ => {}
            }

            if val? == 0 {
                return Ok(())
            }

            pty.write(&mut clientbuf).await?;
            c.bash_log.push(byte as char);
        }
        }
    }
}
