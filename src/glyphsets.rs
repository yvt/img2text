use crate::Fragment;

/// A set of output glyphs (string fragments) that are associated with
/// expected input image patterns.
///
/// **This trait's methods are exempt from the API stability guarantee.**
pub trait GlyphSet {
    fn mask_dims(&self) -> [usize; 2];
    fn fragment_to_glyph(&self, fragment: Fragment) -> &str;
    fn max_glyph_len(&self) -> usize;
}

include!(concat!(env!("OUT_DIR"), "/glyphsets.rs"));

struct IndexedGlyphSet {
    mask_dims: [usize; 2],
    max_glyph_len: usize,
    index: &'static [&'static str],
}

impl GlyphSet for IndexedGlyphSet {
    fn mask_dims(&self) -> [usize; 2] {
        self.mask_dims
    }

    fn fragment_to_glyph(&self, fragment: u64) -> &str {
        self.index[fragment as usize]
    }

    fn max_glyph_len(&self) -> usize {
        self.max_glyph_len
    }
}
