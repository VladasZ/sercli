use std::{path::PathBuf, process::Command};

use anyhow::{Result, bail};
use serde::Deserialize;

pub fn git_root() -> Result<PathBuf> {
    let output = Command::new("git").args(["rev-parse", "--show-toplevel"]).output()?;

    if !output.status.success() {
        bail!("Failed to get Git repository root path");
    }

    let git_root = String::from_utf8_lossy(&output.stdout).trim_end_matches('\n').to_string();

    Ok(PathBuf::from(git_root))
}

#[derive(Deserialize)]
struct QwConfig {
    migrations: String,
    compose:    String,
    target_dir: Option<String>,
}

fn find_qw_toml() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let candidate = dir.join("qw.toml");
        if candidate.exists() {
            return Ok(candidate);
        }
        if !dir.pop() {
            bail!("qw.toml not found in current directory or any parent");
        }
    }
}

fn qw_config() -> Result<(QwConfig, PathBuf)> {
    let path = find_qw_toml()?;
    let root = path.parent().unwrap().to_path_buf();
    let content = std::fs::read_to_string(&path)?;
    Ok((toml::from_str(&content)?, root))
}

pub fn migrations_path() -> Result<PathBuf> {
    let (config, root) = qw_config()?;
    Ok(root.join(config.migrations))
}

pub fn compose_path() -> Result<PathBuf> {
    let (config, root) = qw_config()?;
    Ok(root.join(config.compose))
}

pub fn target_dir() -> Result<Option<String>> {
    let (config, _) = qw_config()?;
    Ok(config.target_dir)
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use crate::git_root;

    #[test]
    fn test() -> Result<()> {
        assert_eq!("sercli", git_root()?.iter().last().unwrap());
        Ok(())
    }
}
