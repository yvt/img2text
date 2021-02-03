use clap::Clap;
use std::path::PathBuf;

#[derive(Clap, Debug)]
struct Opts {
    /// The image to process
    pub image_path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("img2text=info"))
        .init();

    let opts: Opts = Clap::parse();
    log::debug!("opts = {:#?}", opts);

    let _img = image::open(&opts.image_path)?;

    log::info!("saluton");

    Ok(())
}
