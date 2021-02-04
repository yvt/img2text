use anyhow::{anyhow, bail, Context, Result};
use clap::{Clap, ValueHint};
use std::{convert::TryInto, io::prelude::*, path::PathBuf, str::FromStr};

mod otsu;

#[derive(Clap, Debug)]
#[clap(long_about = r"
Image-to-text converter

                       🬞🬭🬵🬹🬵█🬱🬹🬱🬭🬏
                    🬭🬷███🬎🬎🬬🬭🬝🬎🬎███🬲🬭
                  🬦🬹██🬴🬮🬭🬭🬭🬭🬭🬭🬭🬭 🬁🬊██🬹🬓
                 🬩🬻██████████████🬺🬓 ██🬺🬚
                🬇🬫█🬱🬜🬄▐███🬲🬭🬭🬭🬷███🬄🬉🬪🬵█🬛🬃
   powered by:  🬇🬬█▌  ▐███🬝🬎🬎████🬱  🬞🬷█🬝🬃
                🬇🬬█🬺🬹🬹🬻███🬲🬭🬭 🬨███🬹🬵🬻██🬝🬃
                 🬉🬬██🬝🬎🬎🬎🬎🬎🬎🬎 🬁🬎🬎🬎🬎🬬██🬝🬄
                  🬉🬊██🬕🬨▌       🬷🬆🬨██🬆🬄
                    🬂🬨🬝🬬██🬹🬹🬹🬹🬹██🬝🬬🬕🬂
                       🬁🬂🬉🬎🬂🬎🬂🬎🬄🬂🬀
")]
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
    /// The output size, measured in character cells or percents (e.g., `80`,
    /// `80x40`, `80x40!`, `-80x40`, `100%`).
    /// [default: downscale to terminal size (if the output is a terminal) or
    /// 100% (otherwise)]
    ///
    ///  - 80: Fit within 80x80 character cells
    ///
    ///  - 80x40: Fit within 80x40 character cells, upscaling as necessary
    ///
    ///  - -80x40: Fit within 80x40 character cells, only downscaling
    ///
    ///  - 80x40!: Fit to 80x40 character cells, not maintaining the aspect
    ///    ratio
    ///
    ///  - 150%: Scale by 150%. The actual output size depends on the glyph set
    ///    being used; for example, `2x3` maps each 2x3 block to one character.
    ///
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
    _1x1,
    _1x2,
    _2x2,
    _2x3,
    Braille,
}

impl Style {
    fn glyph_set(&self) -> &dyn img2text::GlyphSet {
        match self {
            Self::Slc => img2text::GLYPH_SET_SLC,
            Self::Ms2x3 => img2text::GLYPH_SET_MS_2X3,
            Self::_1x1 => img2text::GLYPH_SET_1X1,
            Self::_1x2 => img2text::GLYPH_SET_1X2,
            Self::_2x2 => img2text::GLYPH_SET_2X2,
            Self::_2x3 => img2text::GLYPH_SET_2X3,
            Self::Braille => img2text::GLYPH_SET_BRAILLE8,
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
enum SizeSpec {
    Absolute { dims: [usize; 2], mode: SizeMode },
    Relative(f64),
}

#[derive(Debug, PartialEq)]
enum SizeMode {
    Contain,
    Fill,
    ScaleDown,
}

impl FromStr for SizeSpec {
    type Err = String;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if let Some(rest) = s.strip_suffix("%") {
            let ratio: f64 = rest.parse().map_err(|_| format!("bad ratio: '{}'", rest))?;

            if !ratio.is_finite() || ratio < 0.0 {
                return Err(format!("ratio out of range: '{}'", rest));
            }

            return Ok(Self::Relative(ratio));
        }

        let force = if let Some(rest) = s.strip_suffix("!") {
            s = rest;
            true
        } else {
            false
        };

        let scale_down = if let Some(rest) = s.strip_prefix("-") {
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

        Ok(Self::Absolute {
            dims,
            mode: match (force, scale_down) {
                (true, false) => SizeMode::Fill,
                (false, true) => SizeMode::ScaleDown,
                (false, false) => SizeMode::Contain,
                (true, true) => return Err("cannot specify both `!` and `-`".to_owned()),
            },
        })
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("img2text=info"))
        .init();

    let mut opts: Opts = Clap::parse();
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

    // Resize the image to the terminal size if the size is not specified
    let console_stdout = console::Term::stdout();
    if opts.out_size.is_none() && console_stdout.features().is_attended() {
        if let Some((h, w)) = console_stdout.size_checked() {
            let h = h.saturating_sub(3);
            log::info!(
                "downscaling to `{}x{}` (tty size minus some) because stdout is tty, and `-s` is unspecified",
                w,
                h
            );
            opts.out_size = Some(SizeSpec::Absolute {
                mode: SizeMode::ScaleDown,
                dims: [w as _, h as _],
            });
        }
    }

    // Resize the image if requested
    if let Some(out_size) = &opts.out_size {
        let in_dims = match out_size {
            SizeSpec::Absolute { dims, mode } => {
                let mask_dims = b2t_opts.glyph_set.mask_dims();
                let mask_overlap = b2t_opts.glyph_set.mask_overlap();

                // `out_dims`: measured in character cells
                let out_dims = if *mode == SizeMode::Fill {
                    *dims
                } else {
                    // Calculate the "natural" size
                    let [nat_out_w, nat_out_h] = [
                        img2text::num_glyphs_for_image_width(img.width() as _, &b2t_opts),
                        img2text::num_lines_for_image_height(img.height() as _, &b2t_opts),
                    ];
                    let aspect = (mask_dims[1] - mask_overlap[1]) as f64
                        / (mask_dims[0] - mask_overlap[0]) as f64
                        * opts.cell_width;

                    let [img_w, img_h] = [
                        nat_out_w as f64 / aspect.max(1.0),
                        nat_out_h as f64 * aspect.min(1.0),
                    ];
                    log::debug!("'natural' output size = {:?}", [img_w, img_h]);
                    let scale_x = dims[0] as f64 / img_w;
                    let scale_y = dims[1] as f64 / img_h;

                    let mut scale = f64::min(scale_x, scale_y);
                    if *mode == SizeMode::ScaleDown {
                        scale = scale.min(1.0);
                    }
                    log::debug!("scaling the 'natural' output size by {}...", scale);

                    [
                        (img_w * scale).round() as usize,
                        (img_h * scale).round() as usize,
                    ]
                };

                log::debug!("output size goal = {:?}", out_dims);

                // FIXME: Waiting for `try` blocks
                (|| {
                    Some([
                        out_dims[0]
                            .checked_mul(mask_dims[0] - mask_overlap[0])?
                            .checked_add(mask_overlap[0])?
                            .try_into()
                            .ok()?,
                        out_dims[1]
                            .checked_mul(mask_dims[1] - mask_overlap[1])?
                            .checked_add(mask_overlap[1])?
                            .try_into()
                            .ok()?,
                    ])
                })()
                .ok_or_else(|| anyhow!("requested size is too large"))?
            }
            SizeSpec::Relative(ratio) => {
                let w = img.width() as f64 * ratio;
                let h = img.height() as f64 * ratio;
                if w > u32::MAX as f64 || h > u32::MAX as f64 {
                    bail!("requested size is too large");
                }
                // FIXME: Waiting for `try` blocks
                [w as _, h as _]
            }
        };

        log::debug!(
            "resampling the image from {:?} to {:?}",
            match img.dimensions() {
                (x, y) => [x, y],
            },
            in_dims
        );

        img = image::imageops::resize(
            &img,
            in_dims[0],
            in_dims[1],
            image::imageops::FilterType::CatmullRom,
        );
    }

    log::debug!(
        "expected output size for image of size {:?} is {:?}",
        match img.dimensions() {
            (x, y) => [x, y],
        },
        [
            img2text::num_glyphs_for_image_width(img.width() as _, &b2t_opts),
            img2text::num_lines_for_image_height(img.height() as _, &b2t_opts),
        ]
    );

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
