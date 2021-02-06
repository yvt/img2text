use js_sys::global;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, future::Future, pin::Pin};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

#[derive(PartialEq, Clone)]
pub struct Opts {
    pub image: HtmlImageElement,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerRequest {
    gray_image: Vec<u8>,
    width: usize,
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

pub async fn transform<TWorkerClientInterface: WorkerClientInterface>(
    opts: Opts,
    mut worker: TWorkerClientInterface,
) -> Result<String, TWorkerClientInterface::Error> {
    // Since off-screen canvases are not supported by Safari, we convert the
    // image into raw pixels in a main thread
    let canvas = create_canvas();
    let [width, height] = [opts.image.width(), opts.image.height()];
    canvas.set_width(width);
    canvas.set_height(height);

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
        })
        .await?;

    Ok(response.text)
}

/// Implements a portion of the transformation process that can run in a worker.
pub fn worker_kernel(request: WorkerRequest) -> WorkerResponse {
    // Convert the image to `image::GrayImage`
    let image = image::GrayImage::from_raw(
        request.width as _,
        (request.gray_image.len() / request.width) as u32,
        request.gray_image,
    )
    .unwrap();

    // bmp2text options
    let mut b2t_opts = img2text::Bmp2textOpts::new();
    b2t_opts.glyph_set = img2text::GLYPH_SET_BRAILLE8;

    use img2text::ImageRead;
    let img_proxy = GrayImageRead {
        image: &image,
        threshold: 128, // TODO
        invert: false,  // TODO
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
