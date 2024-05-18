use bomper::config::Config;
use clap::Parser;
use std::path::PathBuf;

mod app;
use app::App;

mod cli;
use cli::{Args, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .init();

    let args = Args::parse();
    tracing::debug!(?args);

    let project = project_base_directory::Project::discover()
        .map_err(|_| bomper::error::Error::ProjectBaseDirectory)?;
    tracing::debug!(?project);
    match args.base_args.repository {
        Some(ref repo) => {
            std::env::set_current_dir(repo)?;
        }
        None => {
            if let Some(base_directory) = project.root_directory {
                std::env::set_current_dir(base_directory)?;
            }
        }
    }

    let config_path = match &args.base_args.config_file {
        Some(path) => path.to_owned(),
        None => {
            let config_path = match project.config_home {
                Some(dir) => {
                    let config_path = dir.join("bomp.ron");
                    if config_path.exists() {
                        config_path
                    } else {
                        PathBuf::from("bomp.ron")
                    }
                }
                None => PathBuf::from("bomp.ron"),
            };
            if !config_path.exists() {
                return Err("No configuration file found".into());
            }
            config_path.clone()
        }
    };
    let config_path = config_path.canonicalize()?;
    tracing::debug!(?config_path);
    let config = Config::from_ron(&config_path)?;
    tracing::debug!(?config);

    let app = App::new(config);
    match args.command {
        Commands::Changelog(changelog) => {
            app.changelog(&changelog)?;
        }
        Commands::RawBump(raw_bump) => {
            app.raw_bump(&raw_bump)?;
        }
        Commands::Bump(bump) => {
            app.bump(&bump)?;
        }
    }
    Ok(())
}
