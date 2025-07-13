use std::{fs::File, io::Write, path::Path};

use anyhow::Result;

use crate::migrations::Migrations;

pub struct Generator {}

impl Generator {
    pub fn run(migrations_path: impl AsRef<Path>) -> Result<()> {
        let migrations_path = migrations_path.as_ref();
        let migrations = Migrations::get(migrations_path)?;

        let migrations_parent = migrations_path.parent().expect("Migrations path has no parent");

        let entities_dir = migrations_parent.join("src/entities");

        if entities_dir.exists() {
            std::fs::remove_dir_all(&entities_dir)?;
        }

        std::fs::create_dir_all(&entities_dir)?;

        let mut mod_file = File::create(entities_dir.join("mod.rs"))?;
        mod_file.write_all(migrations.mod_code()?.as_bytes())?;

        let mut model_file = File::create(entities_dir.join("model.rs"))?;
        model_file.write_all(migrations.model_code()?.as_bytes())?;

        for entity in migrations.entities.values() {
            entity.generate_file(&entities_dir)?;
        }

        for en in migrations.enums.values() {
            en.generate_file(&entities_dir)?;
        }

        Ok(())
    }
}
