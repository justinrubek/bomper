use figment::{Figment, providers::{Format, Toml}};

use bomper::config::Config;

mod app;
use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from(Toml::file("bomp.toml"))?;
    let app = App::custom(config)?;

    println!("{:?}", app.config);
}
