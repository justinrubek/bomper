mod app;
use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app: App = App::new()?;

    app.config.files.iter().for_each(|file| {
        println!("{:?}", file);
    });

    Ok(())
}
