use clap::Clap;
use std::path::PathBuf;

#[derive(Clap, Debug)]
struct Opts {
    /// The image to process
    pub image: PathBuf,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("img2text=info"))
        .init();

    let opts: Opts = Clap::parse();
    log::debug!("opts = {:#?}", opts);

    log::info!("saluton");
}
