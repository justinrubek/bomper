use clap::Parser;

mod app;
use app::App;

mod cli;
use cli::{Args, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let app = App::new(args.base_args).map_err(|_| "Failed to load configuration")?;
    match args.command {
        Commands::RawBump(raw_bump) => {
            app.raw_bump(&raw_bump)?;
        }
        Commands::Bump(bump) => {
            app.bump(&bump)?;
        }
    }
    Ok(())
}
