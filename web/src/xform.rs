use js_sys::global;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, future::Future, pin::Pin};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

#[path = "../../src/otsu.rs"]
mod otsu;

#[derive(PartialEq, Clone)]
pub struct Opts {
    pub image: HtmlImageElement,
    pub max_size: usize,
    pub input_ty: InputTy,
    pub style: Style,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum InputTy {
    /// Automatic detection
    Auto,
    /// White-on-black
    Wob,
    /// Black-on-white
    Bow,
    /// Canny edge detection
    EdgeCanny,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Style {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerRequest {
    gray_image: Vec<u8>,
    width: usize,
    shared_opts: SharedOpts,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerResponse {
    text: String,
}

pub trait WorkerClientInterface {
    /// A custom error type. An implementation of [`Self::request`] can return
    /// a error value of this type. This might be useful to support
    /// cancellation.
    type Error;

    /// Invoke [`worker_kernel`] possibly in a separate thread.
    fn request(
        &mut self,
        req: WorkerRequest,
    ) -> Pin<Box<dyn Future<Output = Result<WorkerResponse, Self::Error>> + '_>>;
}

/// The subset of `Opts` shared by both the main thread and the worker
#[derive(Debug, Serialize, Deserialize)]
struct SharedOpts {
    input_ty: InputTy,
    style: Style,
}

impl SharedOpts {
    fn new(opts: &Opts) -> Self {
        Self {
            input_ty: opts.input_ty,
            style: opts.style,
        }
    }

    fn to_b2t_opts(&self) -> img2text::Bmp2textOpts {
        let mut b2t_opts = img2text::Bmp2textOpts::new();
        b2t_opts.glyph_set = self.style.glyph_set();
        b2t_opts
    }
}

pub async fn transform<TWorkerClientInterface: WorkerClientInterface>(
    opts: Opts,
    mut worker: TWorkerClientInterface,
) -> Result<String, TWorkerClientInterface::Error> {
    let shared_opts = SharedOpts::new(&opts);

    // bmp2text options
    let b2t_opts = shared_opts.to_b2t_opts();

    // Resize the image input to get a output of desired size
    let [width, height] = img2text::adjust_image_size_for_output_size_preserving_aspect_ratio(
        [
            opts.image.natural_width() as _,
            opts.image.natural_height() as _,
        ],
        [opts.max_size, opts.max_size],
        true,
        false, // contain
        0.5,
        &b2t_opts,
    )
    .unwrap();

    // `getImageData`, etc. don't like zero dimensions
    let [width, height] = [width.max(1), height.max(1)];

    // Since off-screen canvases are not supported by Safari, we convert the
    // image into raw pixels in a main thread
    let canvas = create_canvas();
    canvas.set_width(width as _);
    canvas.set_height(height as _);

    let ctx = canvas
        .get_context("2d")
        .unwrap() // should return `Ok(_)`
        .unwrap() // should return non-null
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap(); // should return `CanvasRenderingContext2d`
    ctx.draw_image_with_html_image_element_and_dw_and_dh(
        &opts.image,
        0.0,
        0.0,
        width as f64,
        height as f64,
    )
    .unwrap();

    let image_data = ctx
        .get_image_data(0.0, 0.0, width as f64, height as f64)
        .unwrap();
    let pixels_rgba = image_data.data();

    // Convert it to a grayscale image
    // FIXME: Waiting for `array_chunks`
    let gray_image: Vec<u8> = pixels_rgba
        .0
        .chunks_exact(4)
        .map(|c| (((c[0] as u32 * 5) + (c[1] as u32 * 6) + (c[2] as u32 * 5) + 8) / 16) as u8)
        .collect();

    // Do the rest in a worker
    let response = worker
        .request(WorkerRequest {
            gray_image,
            width: image_data.width() as _,
            shared_opts,
        })
        .await?;

    Ok(response.text)
}

/// Implements a portion of the transformation process that can run in a worker.
pub fn worker_kernel(
    WorkerRequest {
        width,
        mut shared_opts,
        gray_image,
    }: WorkerRequest,
) -> WorkerResponse {
    // Convert the image to `image::GrayImage`
    let mut image =
        image::GrayImage::from_raw(width as _, (gray_image.len() / width) as u32, gray_image)
            .unwrap();

    // Auto-threshold
    let mut histogram = [0; 256];
    otsu::accumulate_histogram(
        &mut histogram,
        image.pixels().map(|&image::Luma([luma])| luma),
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
    let omega0: u32 = histogram[..threshold].iter().sum();
    let omega1: u32 = histogram[threshold..].iter().sum();
    if shared_opts.input_ty == InputTy::Auto {
        let omega_min = omega0.min(omega1);
        let omega_max = omega0.max(omega1);
        log::debug!("[omega_min, omega_max] = {:?}", [omega_min, omega_max]);

        // TODO: probably should take line thickness into account when detecting
        //       line art
        shared_opts.input_ty = if omega_min * 4 > omega_max {
            InputTy::EdgeCanny
        } else {
            if omega1 > omega0 {
                InputTy::Bow
            } else {
                InputTy::Wob
            }
        };
        log::debug!("guessed input_ty = {:?}", shared_opts.input_ty);
    }

    let invert = match shared_opts.input_ty {
        InputTy::Bow => true,
        InputTy::Wob => false,
        InputTy::Auto => unreachable!(),
        InputTy::EdgeCanny => {
            if image.width() != 0 && image.height() != 0 {
                image = imageproc::edges::canny(&image, 10.0, 40.0);
            }
            false
        }
    };

    // bmp2text options
    let b2t_opts = shared_opts.to_b2t_opts();

    use img2text::ImageRead;
    let img_proxy = GrayImageRead {
        image: &image,
        threshold,
        invert,
    };
    let max_out_len =
        if let Some(x) = img2text::max_output_len_for_image_dims(img_proxy.dims(), &b2t_opts) {
            x
        } else {
            return WorkerResponse {
                text: "(output is too large)".to_owned(),
            };
        };
    let mut out_buffer = String::with_capacity(max_out_len);

    img2text::Bmp2text::new()
        .transform_and_write(&img_proxy, &b2t_opts, &mut out_buffer)
        .unwrap();

    WorkerResponse { text: out_buffer }
}

fn create_canvas() -> HtmlCanvasElement {
    let doc = global()
        .unchecked_into::<web_sys::Window>()
        .document()
        .unwrap();

    doc.create_element("canvas")
        .unwrap()
        .unchecked_into::<HtmlCanvasElement>()
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
