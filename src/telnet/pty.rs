// Entire Fucking Wrapper Written Work NON BLOCKING FD
use std::{
    ffi::{CString, c_void},
    io,
    os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd},
    process::{Command, exit},
};

use anyhow::{Result, anyhow};
use nix::{
    fcntl,
    libc::{self, SIGKILL, execvp, size_t},
    pty,
    unistd::Pid,
};
use tokio::io::unix::AsyncFd;


pub struct Asyncpty {
    child: Pid,
    pub fd: AsyncFd<OwnedFd>,
}

impl Drop for Asyncpty {
    // Garbage Collection
    fn drop(&mut self) {
        unsafe {
            libc::kill(self.child.as_raw(), libc::SIGINT);
        }
    }
}

impl Asyncpty {
    fn set_nonblocking(fd: BorrowedFd<'_>) -> Result<()> {
        if fcntl::fcntl(fd, fcntl::F_SETFL(fcntl::OFlag::O_NONBLOCK))? == 1 {
            return Err(anyhow!("failed to set as non blocking"));
        }
        Ok(())
    }

    /// Single non-blocking read using AsyncFd.
    ///
    /// Returns:
    /// - `Ok(Some(n))` → read `n` bytes (0 = EOF)
    /// - `Ok(None)`   → would block / not actually ready
    /// - `Err(e)`     → real I/O error
    ///
    /// Notes:
    /// - Performs exactly one read (no loop)
    /// - FD must be non-blocking
    pub async fn write(&self, buf: &mut [u8]) -> Result<Option<isize>> {
        let mut guard = self.fd.writable().await?;
        match guard.try_io(|inner| {
            let fd = inner.get_ref().as_raw_fd();
            unsafe {
                let res = libc::write(fd, buf.as_mut_ptr() as *mut c_void, buf.len());
                if res < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(res)
            }
        }) {
            /*
               Getting The Buffer Read Count from the closure Above ^^
            */
            Ok(count) => Ok(Some(count?)),
            // Blocking Check
            Err(_) => Ok(None),
        }
    }

    /// Single non-blocking read using AsyncFd.
    ///
    /// Returns:
    /// - `Ok(Some(n))` → read `n` bytes (0 = EOF)
    /// - `Ok(None)`   → would block / not actually ready
    /// - `Err(e)`     → real I/O error
    ///
    /// Notes:
    /// - Performs exactly one read (no loop)
    /// - FD must be non-blocking
    pub async fn read(&self, buf: &mut [u8], count: size_t) -> Result<Option<isize>> {
        let mut guard = self.fd.readable().await?;
        match guard.try_io(|inner| {
            let fd = inner.get_ref().as_raw_fd();
            unsafe {
                let res = libc::read(fd, buf.as_mut_ptr() as *mut c_void, count);
                if res < 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(res)
            }
        }) {
            /*
               Getting The Buffer Read Count from the closure Above ^^
            */
            Ok(count) => Ok(Some(count?)),
            // Blocking Check
            Err(_) => Ok(None),
        }
    }
    pub fn new(bin: String, args: Vec<String>) -> Result<Self> {
        unsafe {
            match pty::forkpty(None, None)? {
                pty::ForkptyResult::Parent { child, master } => {
                    // Manual cleanup
                    if let Err(why) = Self::set_nonblocking(master.as_fd()) {
                        libc::kill(child.into(), SIGKILL);
                        libc::close(master.as_raw_fd());
                        return Err(why);
                    }
                    // Current Process handle return data here
                    return Ok(Self {
                        fd: AsyncFd::new(master)?,
                        child: child,
                    });
                }
                pty::ForkptyResult::Child => {
                    let mut c = Command::new(bin);
                    for arg in args {
                        c.arg(arg);
                    } 
                    
                    exit(1);
                }
            }
        }
    }
}
