mod file;
use std::{io::Read, path::Path};

use file::FileJail;

use crate::{
    config::{Config, FileTableData},
    replacers::{search::SearchReplacer, Replacer},
};

#[test]
fn config_simple() {
    FileJail::expect_with(|jail| {
        jail.create_file(
            "bomp.toml",
            r#"
            [file."Cargo.toml"]
        "#,
        )?;
        let config = Config::from_file(&String::from("bomp.toml"))?;
        let info: Option<&FileTableData> = config.file.get(Path::new("Cargo.toml"));

        if let Some(info) = info {
            assert_eq!(info.search_value, None);
        } else {
            panic!("config.file.get(\"Cargo.toml\") returned None");
        }

        Ok(())
    });
}

#[test]
fn config_search() {
    FileJail::expect_with(|jail| {
        jail.create_file(
            "bomp.toml",
            r#"
            [file."Cargo.toml"]
            search_value = "bomper"
        "#,
        )?;

        let config = Config::from_file(&String::from("bomp.toml"))?;
        let info: Option<&FileTableData> = config.file.get(Path::new("Cargo.toml"));

        if let Some(info) = info {
            assert_eq!(info.search_value, Some(String::from("bomper")));
        } else {
            panic!("config.file.get(\"Cargo.toml\") returned None");
        }

        Ok(())
    });
}

#[test]
fn dual_replace() {
    FileJail::expect_with(|jail| {
        jail.create_file(
            "Cargo.lock",
            r#"
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 3

[[package]]
name = "package1"
version = "0.1.0"
dependencies = [
 "axum",
 "tokio",
 "tracing",
 "tracing-subscriber",
]

[[package]]
name = "package2"
version = "0.1.0"
dependencies = [
 "serde",
 "serde_json",
]

[[package]]
name = "package3"
version = "0.1.0"
dependencies = [
 "tracing",
 "tracing-subscriber",
]"#,
        )?;

        let replacer = SearchReplacer::new(
            Path::new("Cargo.lock").to_path_buf(),
            "0.1.0",
            "package1|package2",
            "0.2.0",
        )?
        .overwrite_file()?;

        if let Some(mut replacer) = replacer {
            let mut replaced_value = String::new();
            replacer.temp_file.read_to_string(&mut replaced_value)?;
            assert_eq!(
                replaced_value,
                r#"
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 3

[[package]]
name = "package1"
version = "0.2.0"
dependencies = [
 "axum",
 "tokio",
 "tracing",
 "tracing-subscriber",
]

[[package]]
name = "package2"
version = "0.2.0"
dependencies = [
 "serde",
 "serde_json",
]

[[package]]
name = "package3"
version = "0.1.0"
dependencies = [
 "tracing",
 "tracing-subscriber",
]"#
            );
        }

        Ok(())
    });
}