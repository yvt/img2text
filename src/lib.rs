mod glyphsets;
mod image;
mod int;
pub use self::{glyphsets::*, image::*};

/// A set of consecutive pixels of a constant length.
///
/// This is currently `u16` but may change to a different unsigned integer type.
pub type Span = u16;

/// The unsigned integer type twice as wide as `Span`.
type Span2 = u32;

// FIXME: Waiting for `T::BITS` (https://github.com/rust-lang/rust/issues/76904)
const SPAN_BITS: usize = <Span as int::BinInteger>::BITS as usize;

/// A small bitmap image, whose dimensions are specified implciitly (e.g., by
/// `GlyphSet::mask_dims`).
pub type Fragment = u64;

#[derive(Clone)]
#[non_exhaustive]
pub struct Bmp2textOpts<'a> {
    pub glyph_set: &'a dyn GlyphSet,
}

impl Default for Bmp2textOpts<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Bmp2textOpts<'_> {
    pub fn new() -> Self {
        Self {
            glyph_set: GLYPH_SET_SLC,
        }
    }
}

/// The working area for bitmap-to-text conversion.
#[derive(Default, Debug)]
pub struct Bmp2text {
    row_group: Vec<Span>,
}

impl Bmp2text {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn transform_and_write(
        &mut self,
        image: &impl ImageRead,
        opts: &Bmp2textOpts,
        out: &mut impl std::fmt::Write,
    ) -> std::fmt::Result {
        use int::BinInteger;

        let glyph_set = opts.glyph_set;
        let mask_dims = glyph_set.mask_dims();
        let mask_overlap = glyph_set.mask_overlap();

        let [img_w, img_h] = image.dims();
        let num_spans_per_line = (img_w + SPAN_BITS - 1) / SPAN_BITS;
        let [out_w, out_h] = [
            num_glyphs_for_image_width(img_w, opts),
            num_lines_for_image_height(img_h, opts),
        ];

        let num_spans_per_line_extra = num_spans_per_line + 1;
        self.row_group
            .resize(num_spans_per_line_extra * mask_dims[1], 0);
        let mut row_group: Vec<&mut [Span]> = self
            .row_group
            .chunks_exact_mut(num_spans_per_line_extra)
            .collect();
        let row_group: &mut [&mut [Span]] = &mut row_group[..mask_dims[1]];

        // The scanning state of each row in `row_group`
        #[derive(Clone, Copy)]
        struct RowState {
            bits: Span2,
        }
        let mut row_states = [RowState { bits: 0 }; 16];
        let row_states = &mut row_states[0..mask_dims[1]];

        for out_y in 0..out_h {
            // Read a row group from the input image
            for (y, row) in row_group.iter_mut().enumerate() {
                image.copy_line_as_spans_to(out_y * (mask_dims[1] - mask_overlap[1]) + y, row);
            }

            let mut num_valid_bits = 0; // .. in `RowState::bits`
            let mut span_i = 0;

            for _ in 0..out_w {
                if num_valid_bits < mask_dims[0] {
                    for (row_state, row) in row_states.iter_mut().zip(row_group.iter_mut()) {
                        row_state.bits |= (row[span_i] as Span2) << num_valid_bits;
                    }
                    span_i += 1;
                    num_valid_bits += SPAN_BITS;
                }

                // Collect an input fragment of dimensions `mask_dims`
                let mut fragment: Fragment = 0;
                for (i, row_state) in row_states.iter_mut().enumerate() {
                    fragment |= (row_state.bits as Fragment & Fragment::ones(0..mask_dims[0] as _))
                        << (i * mask_dims[0]);

                    row_state.bits >>= mask_dims[0] - mask_overlap[0];
                }
                num_valid_bits -= mask_dims[0] - mask_overlap[0];

                debug_assert!(fragment < (1 << (mask_dims[0] * mask_dims[1])));

                // Find the glyph
                let glyph = glyph_set.fragment_to_glyph(fragment);
                out.write_str(glyph)?;
            }
            out.write_str("\n")?;
        }

        Ok(())
    }
}

pub fn num_glyphs_for_image_width(width: usize, opts: &Bmp2textOpts) -> usize {
    let mask_dims = opts.glyph_set.mask_dims();
    let mask_overlap = opts.glyph_set.mask_overlap();
    width.saturating_sub(mask_overlap[0]) / (mask_dims[0] - mask_overlap[0])
}

pub fn num_lines_for_image_height(height: usize, opts: &Bmp2textOpts) -> usize {
    let mask_dims = opts.glyph_set.mask_dims();
    let mask_overlap = opts.glyph_set.mask_overlap();
    height.saturating_sub(mask_overlap[1]) / (mask_dims[1] - mask_overlap[1])
}
/// Calculate the maximum number of bytes possibly outputted by
/// [`Bmp2text::transform_and_write`].
pub fn max_output_len_for_image_dims(
    [width, height]: [usize; 2],
    opts: &Bmp2textOpts,
) -> Option<usize> {
    let glyph_set = opts.glyph_set;
    num_glyphs_for_image_width(width, opts)
        .checked_mul(glyph_set.max_glyph_len())
        .and_then(|x| x.checked_add(1)) // line termination
        .and_then(|x| x.checked_mul(num_lines_for_image_height(height, opts)))
}
