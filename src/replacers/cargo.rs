use anyhow::anyhow;
use cargo_metadata::camino::Utf8Path;
use std::path::Path;
use std::{io::prelude::*, path::PathBuf, str::FromStr};

use super::file::FileReplacer;
use super::VersionReplacement;
use crate::config::CargoReplaceMode;
use crate::error::{Error, Result};
use crate::replacers::Replacer;

/// Replaces all instances of a given value with a new one.
/// This is a somewhat naive implementation, but it works.
/// The area surrounding the value will be checked for matches in the supplied regex
pub struct CargoReplacer {
    lock_path: PathBuf,
    versions: VersionReplacement,
    replace_mode: CargoReplaceMode,
}

impl CargoReplacer {
    pub fn new(versions: VersionReplacement, replace_mode: CargoReplaceMode) -> Result<Self> {
        Ok(Self {
            // TODO: This may need to be specified, or detected
            lock_path: PathBuf::from("Cargo.lock"),
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

        let metadata = get_workspace_metadata()?;
        let workspace_root = &metadata.workspace_root;

        // Read in the file
        let mut lockfile = cargo_lock::Lockfile::load(&self.lock_path)?;

        let packages = match &self.replace_mode {
            CargoReplaceMode::Autodetect => list_cargo_workspace(&metadata)?,
            CargoReplaceMode::Packages(packages) => list_packages(&metadata, packages.clone())?,
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
            (self.lock_path)
                .parent()
                .ok_or_else(|| Error::InvalidPath((self.lock_path).to_path_buf()))?,
        )?;
        let mut file = temp_file.as_file();
        file.write_all(&new_data)?;

        replacers.push(FileReplacer {
            path: self.lock_path.clone(),
            temp_file,
        });

        // Update each package's Cargo.toml with the new version
        for package in packages {
            let replacer =
                update_package(&package, workspace_root, &self.lock_path, &self.versions)?;
            if let Some(replacer) = replacer {
                replacers.push(replacer);
            };
        }

        // Now, we need to update the Cargo.toml in the workspace root
        // This can be a bit more complicated, because the workspace root may be one of the packages
        // (specifically if there is a single package in the workspace)
        // In this case we need to find the package that is the workspace root, and if we're already
        // updating that package

        // First, check to see if we've updated the `Cargo.toml` in the workspace root
        let root_toml_path = workspace_root.join("Cargo.toml");
        let found_workspace_root = replacers
            .iter()
            .find(|replacer| replacer.path == root_toml_path);
        match found_workspace_root {
            Some(_) => {
                // We've already updated the workspace root, so we don't need to do anything
            }
            None => {
                // We haven't updated the workspace root, so we need to do that now
                let replacer = update_workspace_root(workspace_root, &self.versions)?;
                if let Some(replacer) = replacer {
                    replacers.push(replacer);
                };
            }
        }

        Ok(Some(replacers))
    }
}

/// Returns all packages in the cargo workspace
/// This is only the packages that are defined in the local workspace, and not the dependencies
fn list_cargo_workspace(
    metadata: &cargo_metadata::Metadata,
) -> Result<Vec<cargo_metadata::Package>> {
    Ok(metadata.packages.clone())
}

/// Returns all packages in the cargo workspace that match the given name
fn list_packages(
    metadata: &cargo_metadata::Metadata,
    names: Vec<String>,
) -> Result<Vec<cargo_metadata::Package>> {
    let packages = metadata
        .clone()
        .packages
        .into_iter()
        .filter(|package| names.contains(&package.name))
        .collect();

    Ok(packages)
}

/// Retrieves the metadata for the current workspace.
fn get_workspace_metadata() -> Result<cargo_metadata::Metadata> {
    let mut metadata_cmd = cargo_metadata::MetadataCommand::new();
    metadata_cmd.features(cargo_metadata::CargoOpt::AllFeatures);
    metadata_cmd.no_deps();

    let metadata = metadata_cmd.exec()?;

    Ok(metadata)
}

/// Updates the workspace root's Cargo.toml with the new version
fn update_workspace_root(
    workspace_root: &Utf8Path,
    versions: &VersionReplacement,
) -> Result<Option<FileReplacer>> {
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let cargo_toml_path = cargo_toml_path.strip_prefix(workspace_root)?;
    let cargo_toml_content = std::fs::read(cargo_toml_path)?;

    let mut cargo_toml = cargo_toml::Manifest::from_slice(&cargo_toml_content)?;

    let workspace = match cargo_toml.workspace {
        Some(ref mut workspace) => workspace,
        None => return Ok(None),
    };

    let workspace_package = match workspace.package {
        Some(ref mut package) => package,
        None => return Ok(None),
    };

    if workspace_package.version != Some(versions.old_version.clone()) {
        return Ok(None);
    } else {
        workspace_package.version = Some(versions.new_version.clone());
    }

    let temp_file = tempfile::NamedTempFile::new_in(
        (workspace_root)
            .parent()
            .ok_or_else(|| Error::Other(anyhow!("Invalid path: {:?}", workspace_root)))?,
    )?;
    let mut file = temp_file.as_file();

    let data = toml::to_string(&cargo_toml)?;
    file.write_all(data.as_bytes())?;

    Ok(Some(FileReplacer {
        path: cargo_toml_path.into(),
        temp_file,
    }))
}

/// Updates a package's Cargo.toml with the new version
fn update_package(
    package: &cargo_metadata::Package,
    workspace_root: &Utf8Path,
    lock_path: &Path,
    versions: &VersionReplacement,
) -> Result<Option<FileReplacer>> {
    let cargo_toml_path = package.manifest_path.clone();
    let cargo_toml_path = cargo_toml_path.strip_prefix(workspace_root)?;
    let cargo_toml_content = std::fs::read(cargo_toml_path)?;

    let mut cargo_toml = cargo_toml::Manifest::from_slice(&cargo_toml_content)?;
    // let mut cargo_toml = cargo_toml::Manifest::from_path(&cargo_toml_path)?;

    {
        let toml_package = match cargo_toml.package {
            Some(ref mut package) => package,
            None => return Err(Error::InvalidCargoToml(cargo_toml_path.into())),
        };

        let file_version = match &mut toml_package.version {
            // If the version is inherited, we don't need to do anything
            cargo_toml::Inheritable::Inherited { .. } => return Ok(None),
            cargo_toml::Inheritable::Set(value) => value,
        };

        if file_version != &versions.old_version {
            return Ok(None);
        }

        file_version.clone_from(&versions.new_version);
    }

    // check if this is a workspace root
    // if it is, we need to update the workspace root's Cargo.toml
    let workspace_root_toml_path = workspace_root.join("Cargo.toml");
    if cargo_toml_path == workspace_root_toml_path {
        modify_workspace_root(&mut cargo_toml, versions)?;
    }

    let temp_file = tempfile::NamedTempFile::new_in(
        (lock_path)
            .parent()
            .ok_or_else(|| Error::InvalidPath((lock_path).to_path_buf()))?,
    )?;
    let mut file = temp_file.as_file();

    let data = toml::to_string(&cargo_toml)?;
    file.write_all(data.as_bytes())?;

    Ok(Some(FileReplacer {
        path: cargo_toml_path.into(),
        temp_file,
    }))
}

fn modify_workspace_root(
    cargo_toml: &mut cargo_toml::Manifest,
    versions: &VersionReplacement,
) -> Result<()> {
    let workspace = match cargo_toml.workspace {
        Some(ref mut workspace) => workspace,
        None => return Ok(()),
    };

    let workspace_package = match workspace.package {
        Some(ref mut package) => package,
        None => return Ok(()),
    };

    if workspace_package.version != Some(versions.old_version.clone()) {
        return Ok(());
    } else {
        workspace_package.version = Some(versions.new_version.clone());
    }

    Ok(())
}
