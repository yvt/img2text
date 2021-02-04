use anyhow::{anyhow, bail, Context, Result};
use clap::{Clap, ValueHint};
use std::{convert::TryInto, io::prelude::*, path::PathBuf, str::FromStr};

mod otsu;

#[derive(Clap, Debug)]
struct Opts {
    /// The image to process
    #[clap(name = "FILE", value_hint = ValueHint::AnyPath)]
    image_path: PathBuf,
    /// The glyph set to use
    #[clap(short = 'g', default_value = "slc", arg_enum)]
    style: Style,
    /// The width of output characters, only used when `-s` is given without
    /// `!`
    #[clap(short = 'w', default_value = "0.45")]
    cell_width: f64,
    /// The output size. 80 => fit within 80x80; 80x40 => fit within 80x40;
    /// 80x40! => fit to 80x40, not maintaining the aspect ratio
    #[clap(short = 's')]
    out_size: Option<SizeSpec>,

    /// Specifies how to interpret the input image.
    #[clap(short = 'i', default_value = "auto", arg_enum)]
    input_ty: InputTy,
    /// A parameter for the Canny edge detector (`-i edge-canny`).
    ///
    /// Edges with a strength higher than the low threshold will appear in the
    /// output image if there are strong edges nearby.
    #[clap(long = "canny-low-threshold", default_value = "10")]
    edge_canny_low_threshold: f32,
    /// A parameter for the Canny edge detector (`-i edge-canny`).
    ///
    /// Edges with a strength higher than the high threshold will always appear
    /// as edges in the output image.
    #[clap(long = "canny-high-threshold", default_value = "20")]
    edge_canny_high_threshold: f32,
}

#[derive(Clap, Debug)]
enum Style {
    Slc,
    Ms2x3,
    Blocks2x2,
    Blocks2x3,
    Braille2x4,
}

impl Style {
    fn glyph_set(&self) -> &dyn img2text::GlyphSet {
        match self {
            Self::Slc => img2text::GLYPH_SET_SLC,
            Self::Ms2x3 => img2text::GLYPH_SET_MS_2X3,
            Self::Blocks2x2 => img2text::GLYPH_SET_2X2,
            Self::Blocks2x3 => img2text::GLYPH_SET_2X3,
            Self::Braille2x4 => img2text::GLYPH_SET_BRAILLE8,
        }
    }
}

#[derive(Clap, Debug)]
enum InputTy {
    /// Automatic detection
    Auto,
    /// White-on-black
    Wob,
    /// Black-on-white
    Bow,
    /// Canny edge detection
    EdgeCanny,
}

#[derive(Debug)]
struct SizeSpec {
    dims: [usize; 2],
    force: bool,
}

impl FromStr for SizeSpec {
    type Err = String;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let force = if let Some(rest) = s.strip_suffix("!") {
            s = rest;
            true
        } else {
            false
        };

        let dims = if let Some(i) = s.find("x") {
            // width x height
            let width = &s[0..i];
            let height = &s[i + 1..];
            [
                width
                    .parse()
                    .map_err(|_| format!("bad width: '{}'", width))?,
                height
                    .parse()
                    .map_err(|_| format!("bad height: '{}'", height))?,
            ]
        } else {
            // size
            let size = s.parse().map_err(|_| format!("bad size: '{}'", s))?;
            [size, size]
        };

        Ok(Self { dims, force })
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("img2text=info"))
        .init();

    let opts: Opts = Clap::parse();
    log::debug!("opts = {:#?}", opts);

    // Open the image
    let img = image::open(&opts.image_path).with_context(|| {
        format!(
            "Failed to read an input image from '{}'",
            opts.image_path.display()
        )
    })?;
    let mut img = img.into_luma8();

    // Options
    let mut b2t_opts = img2text::Bmp2textOpts::new();
    b2t_opts.glyph_set = opts.style.glyph_set();

    if !opts.cell_width.is_finite() || opts.cell_width <= 0.1 || opts.cell_width > 10.0 {
        bail!("cell_width is out of range");
    }

    if !opts.edge_canny_low_threshold.is_finite()
        || opts.edge_canny_low_threshold <= 0.0
        || opts.edge_canny_low_threshold > 1150.0
    {
        bail!("edge_canny_low_threshold is out of range");
    }

    if !opts.edge_canny_high_threshold.is_finite()
        || opts.edge_canny_high_threshold <= 0.0
        || opts.edge_canny_high_threshold > 1150.0
    {
        bail!("edge_canny_high_threshold is out of range");
    }

    if opts.edge_canny_low_threshold > opts.edge_canny_high_threshold {
        bail!("edge_canny_low_threshold mustn't be greater than edge_canny_high_threshold");
    }

    // Resize the image if requested
    if let Some(out_size) = &opts.out_size {
        let new_dims = if out_size.force {
            out_size.dims
        } else {
            let [img_w, img_h] = [img.width() as f64, img.height() as f64 * opts.cell_width];
            let scale_x = out_size.dims[0] as f64 / img_w;
            let scale_y = out_size.dims[1] as f64 / img_h;

            if scale_x < scale_y {
                [out_size.dims[0], (img_h * scale_x).round() as usize]
            } else {
                [(img_w * scale_y).round() as usize, out_size.dims[1]]
            }
        };

        let mask_dims = b2t_opts.glyph_set.mask_dims();
        let mask_overlap = b2t_opts.glyph_set.mask_overlap();

        // FIXME: Waiting for `try` blocks
        let in_dims = (|| {
            Some([
                new_dims[0]
                    .checked_mul(mask_dims[0] - mask_overlap[0])?
                    .checked_add(mask_overlap[0])?
                    .try_into()
                    .ok()?,
                new_dims[1]
                    .checked_mul(mask_dims[1] - mask_overlap[1])?
                    .checked_add(mask_overlap[1])?
                    .try_into()
                    .ok()?,
            ])
        })()
        .ok_or_else(|| anyhow!("requested size is too large"))?;

        img = image::imageops::resize(
            &img,
            in_dims[0],
            in_dims[1],
            image::imageops::FilterType::CatmullRom,
        );
    }

    // Auto-threshold
    let mut histogram = [0; 256];
    otsu::accumulate_histogram(
        &mut histogram,
        img.pixels().map(|&image::Luma([luma])| luma),
    );
    log::trace!("histogram = {:?}", histogram);
    let threshold = if let Some(x) = otsu::find_threshold(&histogram) {
        log::debug!("threshold = {}", x);
        x
    } else {
        log::debug!("couldn't find the threshold, using the default value 128");
        128
    };

    // black-on-white/white-on-black detection
    let invert = match opts.input_ty {
        InputTy::Bow => true,
        InputTy::Wob => false,
        InputTy::Auto => {
            let omega0: u32 = histogram[..threshold].iter().sum();
            let omega1: u32 = histogram[threshold..].iter().sum();
            omega1 > omega0
        }
        InputTy::EdgeCanny => {
            img = imageproc::edges::canny(
                &img,
                opts.edge_canny_low_threshold,
                opts.edge_canny_high_threshold,
            );
            false
        }
    };

    // Process the image
    use img2text::ImageRead;
    let img_proxy = GrayImageRead {
        image: &img,
        threshold,
        invert,
    };
    let mut out_buffer = String::with_capacity(
        img2text::max_output_len_for_image_dims(img_proxy.dims(), &b2t_opts)
            .ok_or_else(|| anyhow!("image is too large"))?,
    );

    img2text::Bmp2text::new()
        .transform_and_write(&img_proxy, &b2t_opts, &mut out_buffer)
        .unwrap();

    std::io::stdout()
        .write(out_buffer.as_bytes())
        .with_context(|| "Failed to write the output to the standard output")?;

    Ok(())
}

struct GrayImageRead<'a> {
    image: &'a image::GrayImage,
    threshold: usize,
    invert: bool,
}

impl img2text::ImageRead for GrayImageRead<'_> {
    fn dims(&self) -> [usize; 2] {
        let (w, h) = self.image.dimensions();
        [w.try_into().unwrap(), h.try_into().unwrap()]
    }

    fn copy_line_as_spans_to(&self, y: usize, out: &mut [img2text::Span]) {
        let Self {
            image,
            threshold,
            invert,
        } = *self;
        img2text::set_spans_by_fn(out, self.dims()[0], move |x| {
            (image[(x as u32, y as u32)].0[0] as usize >= threshold) ^ invert
        });
    }
}
