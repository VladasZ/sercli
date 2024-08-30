use std::future::Future;

use anyhow::{bail, Result};
use tokio::sync::oneshot::{channel, Sender};

#[derive(Debug)]
pub struct ServerHandle {
    sender: Sender<()>,
}

impl ServerHandle {
    pub fn new() -> (Self, impl Future<Output = ()>) {
        let (sender, rc) = channel::<()>();

        let rc = async {
            rc.await.expect("Failed to receive shutdown signal");
        };

        (Self { sender }, rc)
    }
}

impl ServerHandle {
    pub fn shutdown(self) -> Result<()> {
        if let Err(()) = self.sender.send(()) {
            bail!("Failed to send shutdown signal")
        }
        Ok(())
    }
}