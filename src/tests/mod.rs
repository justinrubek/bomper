mod file;
use std::{io::Read, path::Path};

use file::FileJail;

use crate::{
    config::{CargoReplaceMode, Config, FileTableData},
    replacers::{cargo::CargoReplacer, search::SearchReplacer, Replacer, VersionReplacement},
};

#[test]
fn config_simple() {
    FileJail::expect_with(|jail| {
        jail.create_file(
            "bomp.ron",
            r#"(
                by_file: Some({
                   "Cargo.toml": ( search_value: None, ),
                }),
            )
        "#,
        )?;
        let config = Config::from_ron(&String::from("bomp.ron"))?;
        let by_file = config.by_file.unwrap();
        let info: Option<&FileTableData> = by_file.get(Path::new("Cargo.toml"));

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
            "bomp.ron",
            r#"(
                by_file: Some({
                   "Cargo.toml": ( search_value: Some("bomper"), ),
                }),
            )
        "#,
        )?;

        let config = Config::from_ron(&String::from("bomp.ron"))?;
        let by_file = config.by_file.unwrap();
        let info: Option<&FileTableData> = by_file.get(Path::new("Cargo.toml"));

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
        .determine_replacements()?;

        if let Some(mut replacer) = replacer {
            let mut replacer = replacer.pop().unwrap();

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
        } else {
            panic!("replacer is not generated");
        }

        Ok(())
    });
}

#[test]
/// Replaces a single crate workspace using the cargo replacer with auto-detection
/// This should update both the Cargo.toml and Cargo.lock files
fn cargo_individual() {
    FileJail::expect_with(|jail| {
        jail.create_file(
            "Cargo.toml",
            r#"
[package]
name = "package1"
edition = "2018"
version = "0.1.0"

[dependencies]
package2 = "0.1.0"
package3 = "0.1.0"
"#,
        )?;

        let expected_toml = r#"[package]
name = "package1"
edition = "2018"
version = "0.2.0"

[dependencies]
package2 = "0.1.0"
package3 = "0.1.0"
"#;

        jail.create_file(
            "src/main.rs",
            r#"fn main() {
    println!("Hello, world!");
}"#,
        )?;

        jail.create_file(
            "Cargo.lock",
            r#"# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 3

[[package]]
name = "package1"
version = "0.1.0"
dependencies = [
 "package2",
 "package3",
]

[[package]]
name = "package2"
version = "0.1.0"

[[package]]
name = "package3"
version = "0.1.0"
"#,
        )?;

        let expected_lock = r#"# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 3

[[package]]
name = "package1"
version = "0.2.0"
dependencies = [
 "package2",
 "package3",
]

[[package]]
name = "package2"
version = "0.1.0"

[[package]]
name = "package3"
version = "0.1.0"
"#;

        let version_replacement = VersionReplacement {
            old_version: "0.1.0".to_string(),
            new_version: "0.2.0".to_string(),
        };

        let replacers = CargoReplacer::new(version_replacement, CargoReplaceMode::Autodetect)?
            .determine_replacements()?;

        for replacer in replacers {
            for replacer in replacer {
                let file = replacer.temp_file.path();
                let file_contents = std::fs::read_to_string(file)?;

                if replacer.path.ends_with("Cargo.toml") {
                    assert_eq!(file_contents, expected_toml);
                } else if replacer.path.ends_with("Cargo.lock") {
                    assert_eq!(file_contents, expected_lock);
                } else {
                    panic!("Unexpected file path: {}", replacer.path.display());
                }
            }
        }

        Ok(())
    });
}
