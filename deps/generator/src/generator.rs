use std::{fs::File, io::Write};

use anyhow::Result;
use sercli_utils::git_root;

use crate::migrations::Migrations;

pub struct Generator {}

impl Generator {
    pub fn run() -> Result<()> {
        let migrations = Migrations::get()?;

        let entities_dir = git_root()?.join("model/src/entities");

        if entities_dir.exists() {
            std::fs::remove_dir_all(&entities_dir)?;
        }

        std::fs::create_dir_all(&entities_dir)?;

        let mut mod_file = File::create(entities_dir.join("mod.rs"))?;

        mod_file.write_all(migrations.mod_code().as_bytes())?;

        for entity in migrations.model.values() {
            entity.generate_file(&entities_dir)?;
        }

        Ok(())
    }
}

#[test]
fn generator() -> Result<()> {
    Generator::run()?;

    Ok(())
}
