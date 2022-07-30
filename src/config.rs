use std::collections::HashSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use figment::{Error, Figment, Metadata, Profile, Provider};

#[derive(Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct Config {
    pub files: HashSet<PathBuf>,
}

impl Config {
    pub fn from<T: Provider>(provider: T) -> Result<Config, Error> {
        Figment::from(provider).extract()
    }

    pub fn figment() -> Figment {
        use figment::providers::{Format, Toml};

        Figment::from(Config::default()).merge(Toml::file("bomp.toml"))
    }
}

use figment::value::{Dict, Map};

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("bomper config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }

    fn profile(&self) -> Option<Profile> {
        None
    }
}
