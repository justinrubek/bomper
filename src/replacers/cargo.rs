use std::{io::prelude::*, path::PathBuf, str::FromStr};

use super::VersionReplacement;
use super::file::FileReplacer;
use crate::config::CargoReplaceMode;
use crate::error::{Error, Result};
use crate::replacers::Replacer;

/// Replaces all instances of a given value with a new one.
/// This is a somewhat naive implementation, but it works.
/// The area surrounding the value will be checked for matches in the supplied regex
pub struct CargoReplacer {
    path: PathBuf,
    versions: VersionReplacement,
    replace_mode: CargoReplaceMode,
}

impl CargoReplacer {
    pub fn new(
        versions: VersionReplacement,
        replace_mode: CargoReplaceMode,
    ) -> Result<Self> {
        Ok(Self {
            // TODO: This may need to be specified, or detected
            path: PathBuf::from("Cargo.lock"),
            versions,
            replace_mode,
        })
    }
}

impl Replacer for CargoReplacer {
    /// Replaces all instances of the old_content with the new_content in the file.
    /// Returns a FileReplacer object that can be used to replace the file by persisting the changes.
    fn determine_replacements(self) -> Result<Option<Vec<FileReplacer>>> {
        let mut replacers = Vec::new();

        // Read in the file
        let mut lockfile = cargo_lock::Lockfile::load(&self.path)?;

        let packages = match &self.replace_mode {
            CargoReplaceMode::Autodetect => list_cargo_workspace()?,
            CargoReplaceMode::Packages(packages) => list_packages(packages.clone())?,
        };

        let new_version = cargo_lock::Version::from_str(&self.versions.new_version)?;
        let old_version = cargo_lock::Version::from_str(&self.versions.old_version)?;

        let package_names = packages
            .iter()
            .map(|package| package.name.as_str().to_string())
            .collect::<Vec<String>>();
        
        // update cargo.lock with new versions of packages
        lockfile.packages.iter_mut().for_each(|package| {
            let package_name = package.name.as_str().to_string();

            if package_names.contains(&package_name) && package.version == old_version {
                package.version = new_version.clone();
            }
        });

        let new_data = lockfile.to_string().into_bytes();

        let temp_file = tempfile::NamedTempFile::new_in(
            (self.path)
                .parent()
                .ok_or_else(|| Error::InvalidPath((self.path).to_path_buf()))?,
        )?;
        let mut file = temp_file.as_file();
        file.write_all(&new_data)?;

        replacers.push(FileReplacer {
            path: self.path.clone(),
            temp_file,
        });

        // Update each package's Cargo.toml with the new version
        for package in packages {
            let cargo_toml_path = package.manifest_path;

            let mut cargo_toml = cargo_toml::Manifest::from_path(&cargo_toml_path)?;

            let toml_package = match cargo_toml.package {
                Some(ref mut package) => package,
                None => return Err(Error::InvalidCargoToml(cargo_toml_path)),
            };

            let file_version = match &mut toml_package.version {
                cargo_toml::Inheritable::Inherited { .. } => continue,
                cargo_toml::Inheritable::Set(value) => value,
            };

            if file_version != &self.versions.old_version {
                continue;
            }

            *file_version = self.versions.new_version.clone();

            let temp_file = tempfile::NamedTempFile::new_in(
                (self.path)
                    .parent()
                    .ok_or_else(|| Error::InvalidPath((self.path).to_path_buf()))?,
            )?;
            let mut file = temp_file.as_file();

            let data = toml::to_string(&cargo_toml)?;
            file.write_all(data.as_bytes())?;

            replacers.push(FileReplacer {
                path: cargo_toml_path.into(),
                temp_file,
            });
        }

        Ok(Some(replacers))
    }
}

/// Returns all packages in the cargo workspace
/// This is only the packages that are defined in the local workspace, and not the dependencies
fn list_cargo_workspace() -> Result<Vec<cargo_metadata::Package>> {
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.features(cargo_metadata::CargoOpt::AllFeatures);
    metadata_cmd.no_deps();

    let metadata = metadata_cmd.exec()?;

    Ok(metadata.packages)
}

/// Returns all packages in the cargo workspace that match the given name
fn list_packages(names: Vec<String>) -> Result<Vec<cargo_metadata::Package>> {
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.features(cargo_metadata::CargoOpt::AllFeatures);
    metadata_cmd.no_deps();

    let metadata = metadata_cmd.exec()?;

    let packages = metadata
        .packages
        .into_iter()
        .filter(|package| names.contains(&package.name))
        .collect();

    Ok(packages)
}
