use std::{path::PathBuf, process::Command};

use anyhow::{Result, bail};

pub fn git_root() -> Result<PathBuf> {
    let output = Command::new("git").args(["rev-parse", "--show-toplevel"]).output()?;

    if !output.status.success() {
        bail!("Failed to get Git repository root path");
    }

    let git_root = String::from_utf8_lossy(&output.stdout).trim_end_matches('\n').to_string();

    Ok(PathBuf::from(git_root))
}

#[cfg(test)]
mod test {
    use std::env;

    use anyhow::{Result, anyhow};

    use crate::git_root;

    #[test]
    fn test() -> Result<()> {
        assert_eq!("sercli", git_root()?.iter().last().unwrap());

        let original_dir = env::current_dir()?;
        let home_dir = home::home_dir().ok_or(anyhow!("No HOME"))?;

        env::set_current_dir(&home_dir)?;

        let result = git_root();

        env::set_current_dir(original_dir)?;

        assert_eq!(
            anyhow!("Failed to get Git repository root path").to_string(),
            result.err().unwrap().to_string()
        );

        Ok(())
    }
}
