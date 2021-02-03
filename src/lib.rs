mod image;
pub use self::image::*;

/// A set of consecutive pixels of a constant length.
///
/// This is currently `u16` but may change to a different unsigned integer type.
pub type Span = u16;

// FIXME: Waiting for `T::BITS` (https://github.com/rust-lang/rust/issues/76904)
const SPAN_BITS: usize = std::mem::size_of::<Span>() * 8;

/// The working area for bitmap-to-text conversion.
#[derive(Debug)]
pub struct Bmp2text {}

impl Default for Bmp2text {
    fn default() -> Self {
        Self::new()
    }
}

impl Bmp2text {
    pub fn new() -> Self {
        Self {}
    }

    pub fn transform_and_write(
        &mut self,
        image: &impl ImageRead,
        out: &mut impl std::fmt::Write,
    ) -> std::fmt::Result {
        // dummy impl
        let mut spans: Vec<Span> = vec![0; (image.dims()[0] + SPAN_BITS - 1) / SPAN_BITS];
        for y in 0..image.dims()[1] {
            image.copy_line_as_spans_to(y, &mut spans);
            for x in 0..image.dims()[0] {
                out.write_str(
                    [" ", "*"][(spans[x / SPAN_BITS] & (1 << (x % SPAN_BITS)) != 0) as usize],
                )?;
            }
            out.write_str("\n")?;
        }
        Ok(())
    }
}

/// Calculate the maximum number of bytes per line (excluding the line
/// terminator `"\n"`) possibly outputted by [`Bmp2text::transform_and_write`].
pub fn max_output_line_len_for_image_width(width: usize) -> Option<usize> {
    Some(width)
}

/// Calculate the maximum number of bytes per line (excluding the line
/// terminator `"\n"`) possibly outputted by [`Bmp2text::transform_and_write`].
pub fn max_output_len_for_image_dims(dims: [usize; 2]) -> Option<usize> {
    max_output_line_len_for_image_width(dims[0])
        .and_then(|x| x.checked_add(1))
        .and_then(|x| x.checked_mul(dims[1]))
}
