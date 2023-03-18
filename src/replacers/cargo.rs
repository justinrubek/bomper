use std::{io::prelude::*, path::PathBuf, str::FromStr};

use super::VersionReplacement;
use super::file::FileReplacer;
use crate::config::CargoLockReplaceMode;
use crate::error::{Error, Result};
use crate::replacers::Replacer;

/// Replaces all instances of a given value with a new one.
/// This is a somewhat naive implementation, but it works.
/// The area surrounding the value will be checked for matches in the supplied regex
pub struct CargoLockReplacer {
    path: PathBuf,
    versions: VersionReplacement,
    replace_mode: CargoLockReplaceMode,
}

impl CargoLockReplacer {
    pub fn new(
        versions: VersionReplacement,
        replace_mode: CargoLockReplaceMode,
    ) -> Result<Self> {
        Ok(Self {
            // TODO: This may need to be specified, or detected
            path: PathBuf::from("Cargo.lock"),
            versions,
            replace_mode,
        })
    }
}

impl Replacer for CargoLockReplacer {
    /// Replaces all instances of the old_content with the new_content in the file.
    /// Returns a FileReplacer object that can be used to replace the file by persisting the changes.
    fn overwrite_file(self) -> Result<Option<FileReplacer>> {
        // Read in the file
        let mut lockfile = cargo_lock::Lockfile::load(&self.path)?;

        let package_names = match &self.replace_mode {
            CargoLockReplaceMode::Autodetect => list_cargo_workspace()?,
            CargoLockReplaceMode::Packages(packages) => packages.to_vec(),
        };

        let new_version = cargo_lock::Version::from_str(&self.versions.new_version)?;
        let old_version = cargo_lock::Version::from_str(&self.versions.old_version)?;
        
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

        Ok(Some(FileReplacer {
            path: self.path,
            temp_file,
        }))
    }
}

/// Returns the names of all packages in the cargo workspace
/// This is only the packages that are defined in the local workspace, and not the dependencies
fn list_cargo_workspace() -> Result<Vec<String>> {
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.features(cargo_metadata::CargoOpt::AllFeatures);
    metadata_cmd.no_deps();

    let metadata = metadata_cmd.exec()?;

    let package_names = metadata.packages
        .iter()
        .map(|package| package.name.clone())
        .collect::<Vec<String>>();

    Ok(package_names)
}
