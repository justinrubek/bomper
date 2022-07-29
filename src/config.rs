use serde::{Deserialize, Serialize};

use figment::{Figment, Provider, Error, Metadata, Profile};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Config {
    pub files: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            files: vec![],
        }
    }
}

impl Config {
    fn from<T: Provider>(provider: T) -> Result<Config, Error> {
        Figment::from(provider).extract()
    }

    fn figment() -> Figment {
        use figment::providers::Env;

        Figment::from(Config::default()).merge(Env::prefixed("BOMPER_"))
    }
}

use figment::value::{Map, Dict};

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
