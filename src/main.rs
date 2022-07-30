use clap::Parser;

mod app;
use app::App;

mod cli;
use cli::Args;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app: App = App::new()?;
    let args = Args::parse();

    app.run(&args)?;

    Ok(())
}
