use std::future::Future;

use anyhow::{Result, bail};
use tokio::sync::oneshot::{Sender, channel};

#[derive(Debug)]
pub struct ServerHandle {
    sender: Sender<()>,
}

impl ServerHandle {
    pub fn new() -> (Self, impl Future<Output = ()>) {
        let (sender, rc) = channel::<()>();

        let rc = async {
            let result = rc.await;

            result.expect(
                "Failed to receive server shutdown signal. Was ServerHandle dropped before server shutdown?",
            );
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
