use clap::Parser;

mod app;
use app::App;

mod cli;
use cli::Args;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let app: App = App::new(args)?;
    app.run()?;

    Ok(())
}
