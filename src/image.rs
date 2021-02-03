//! Input image representation
use crate::{Span, SPAN_BITS};

pub trait ImageRead {
    /// Get the image's dimensions as `[width, height]`.
    fn dims(&self) -> [usize; 2];

    /// Convert the specified row to `Span`s.
    ///
    /// `out` contains at least `(self.dims()[0] + Span::BITS - 1) / Span::BITS`
    /// elements.
    ///
    /// [`set_spans_by_fn`] may be useful to implement this method.
    fn copy_line_as_spans_to(&self, y: usize, out: &mut [Span]);
}

/// Fill `[Span]` using a given function that returns the desired value for each
/// bit.
pub fn set_spans_by_fn(
    out_spans: &mut [Span],
    num_pixels: usize,
    mut f: impl FnMut(usize) -> bool,
) {
    let mut i = 0;
    for out_span in out_spans[..num_pixels / SPAN_BITS].iter_mut() {
        let mut b = 0;
        for k in 0..SPAN_BITS {
            b |= (f(i) as Span) << k;
            i += 1;
        }
        *out_span = b;
    }

    if num_pixels % SPAN_BITS != 0 {
        let mut b = 0;
        for k in 0..num_pixels % SPAN_BITS {
            b |= (f(i) as Span) << k;
            i += 1;
        }
        out_spans[num_pixels / SPAN_BITS] = b;
    }
}
