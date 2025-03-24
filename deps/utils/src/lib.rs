use std::{path::PathBuf, process::Command};

use anyhow::{Result, bail};

pub fn git_root() -> Result<PathBuf> {
    let output = Command::new("git").args(["rev-parse", "--show-toplevel"]).output()?;

    if !output.status.success() {
        bail!("Failed to get Git repository root path");
    }

    assert!(output.status.success(), "Failed to get Git repository root path");
    let git_root = String::from_utf8_lossy(&output.stdout).trim_end_matches('\n').to_string();

    Ok(PathBuf::from(git_root))
}
