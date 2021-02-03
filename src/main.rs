use clap::{Clap, ValueHint};
use std::{convert::TryInto, path::PathBuf};

#[derive(Clap, Debug)]
struct Opts {
    /// The image to process
    #[clap(name = "FILE", value_hint = ValueHint::AnyPath)]
    image_path: PathBuf,
    #[clap(short = 's', default_value = "slc", arg_enum)]
    style: Style,
}

#[derive(Clap, Debug)]
enum Style {
    Slc,
    Slc2,
    Blocks2x2,
    Blocks2x3,
}

impl Style {
    fn glyph_set(&self) -> &dyn img2text::GlyphSet {
        match self {
            Self::Slc => img2text::GLYPH_SET_SLC,
            Self::Slc2 => img2text::GLYPH_SET_SLC2,
            Self::Blocks2x2 => img2text::GLYPH_SET_2X2,
            Self::Blocks2x3 => img2text::GLYPH_SET_2X3,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("img2text=info"))
        .init();

    let opts: Opts = Clap::parse();
    log::debug!("opts = {:#?}", opts);

    // Open the image
    let img = image::open(&opts.image_path)?;
    let img = img.into_luma8();

    // Options
    let mut b2t_opts = img2text::Bmp2textOpts::new();
    b2t_opts.glyph_set = opts.style.glyph_set();

    // Process the image
    use img2text::ImageRead;
    let img_proxy = GrayImageRead(&img);
    let mut out_buffer = String::with_capacity(
        img2text::max_output_len_for_image_dims(img_proxy.dims(), &b2t_opts)
            .expect("image is too large"),
    );

    img2text::Bmp2text::new()
        .transform_and_write(&img_proxy, &b2t_opts, &mut out_buffer)
        .unwrap();

    print!("{}", out_buffer);

    Ok(())
}

struct GrayImageRead<'a>(&'a image::GrayImage);

impl img2text::ImageRead for GrayImageRead<'_> {
    fn dims(&self) -> [usize; 2] {
        let (w, h) = self.0.dimensions();
        [w.try_into().unwrap(), h.try_into().unwrap()]
    }

    fn copy_line_as_spans_to(&self, y: usize, out: &mut [img2text::Span]) {
        let img = self.0;
        img2text::set_spans_by_fn(out, self.dims()[0], move |x| {
            img[(x as u32, y as u32)].0[0] >= 128
        });
    }
}
