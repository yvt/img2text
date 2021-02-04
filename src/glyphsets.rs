use crate::Fragment;

/// A set of output glyphs (string fragments) that are associated with
/// expected input image patterns.
///
/// **This trait's methods are exempt from the API stability guarantee.**
pub trait GlyphSet {
    fn mask_dims(&self) -> [usize; 2];
    fn mask_overlap(&self) -> [usize; 2];
    fn fragment_to_glyph(&self, fragment: Fragment) -> &str;
    fn max_glyph_len(&self) -> usize;
}

include!(concat!(env!("OUT_DIR"), "/glyphsets.rs"));

struct IndexedGlyphSet {
    mask_dims: [usize; 2],
    mask_overlap: [usize; 2],
    max_glyph_len: usize,
    index: &'static [&'static str],
}

impl GlyphSet for IndexedGlyphSet {
    fn mask_dims(&self) -> [usize; 2] {
        self.mask_dims
    }

    fn mask_overlap(&self) -> [usize; 2] {
        self.mask_overlap
    }

    fn fragment_to_glyph(&self, fragment: u64) -> &str {
        self.index[fragment as usize]
    }

    fn max_glyph_len(&self) -> usize {
        self.max_glyph_len
    }
}

pub const GLYPH_SET_BRAILLE8: &dyn GlyphSet = &Braille8GlyphSet(());

struct Braille8GlyphSet(());

impl GlyphSet for Braille8GlyphSet {
    fn mask_dims(&self) -> [usize; 2] {
        [2, 4]
    }

    fn mask_overlap(&self) -> [usize; 2] {
        [0, 0]
    }

    fn fragment_to_glyph(&self, fragment: u64) -> &str {
        let pats = "⠀⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠋⠌⠍⠎⠏\
            ⠐⠑⠒⠓⠔⠕⠖⠗⠘⠙⠚⠛⠜⠝⠞⠟\
            ⠠⠡⠢⠣⠤⠥⠦⠧⠨⠩⠪⠫⠬⠭⠮⠯\
            ⠰⠱⠲⠳⠴⠵⠶⠷⠸⠹⠺⠻⠼⠽⠾⠿\
            ⡀⡁⡂⡃⡄⡅⡆⡇⡈⡉⡊⡋⡌⡍⡎⡏\
            ⡐⡑⡒⡓⡔⡕⡖⡗⡘⡙⡚⡛⡜⡝⡞⡟\
            ⡠⡡⡢⡣⡤⡥⡦⡧⡨⡩⡪⡫⡬⡭⡮⡯\
            ⡰⡱⡲⡳⡴⡵⡶⡷⡸⡹⡺⡻⡼⡽⡾⡿\
            ⢀⢁⢂⢃⢄⢅⢆⢇⢈⢉⢊⢋⢌⢍⢎⢏\
            ⢐⢑⢒⢓⢔⢕⢖⢗⢘⢙⢚⢛⢜⢝⢞⢟\
            ⢠⢡⢢⢣⢤⢥⢦⢧⢨⢩⢪⢫⢬⢭⢮⢯\
            ⢰⢱⢲⢳⢴⢵⢶⢷⢸⢹⢺⢻⢼⢽⢾⢿\
            ⣀⣁⣂⣃⣄⣅⣆⣇⣈⣉⣊⣋⣌⣍⣎⣏\
            ⣐⣑⣒⣓⣔⣕⣖⣗⣘⣙⣚⣛⣜⣝⣞⣟\
            ⣠⣡⣢⣣⣤⣥⣦⣧⣨⣩⣪⣫⣬⣭⣮⣯\
            ⣰⣱⣲⣳⣴⣵⣶⣷⣸⣹⣺⣻⣼⣽⣾⣿";
        // ISO/TR 11548-1 dot numbering      Our fragment bit positions:
        // (mapped to bit positions of
        // Unicode code points):
        //
        //             0  3                             0  1
        //             1  4                             2  3
        //             2  5                             4  5
        //             6  7                             6  7
        //
        // Notice that only the positions 1–4 differ between them. Therefore, we
        // use a 16x4-bit LUT to remap these bits.
        const LUT: u64 = {
            let mut lut = 0u64;
            let mut i = 0;
            while i < 16 {
                let b1 = i & 0b0001;
                let b2 = i & 0b0010;
                let b3 = i & 0b0100;
                let b4 = i & 0b1000;
                let uni_b4321 = (b1 << 2) | (b2 >> 1) | (b3 << 1) | (b4 >> 2);
                lut |= uni_b4321 << (i * 4);
                i += 1;
            }
            lut.rotate_left(1)
        };

        let uni_b7650 = fragment & 0b11100001;
        let b4321 = (fragment & 0b00011110) >> 1;
        let uni_b4321 = LUT.rotate_right((b4321 * 4) as _) & 0b11110;
        let uni = uni_b7650 | uni_b4321;
        &pats[uni as usize * 3..uni as usize * 3 + 3]
    }

    fn max_glyph_len(&self) -> usize {
        "⣿".len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn braille8() {
        let gs = GLYPH_SET_BRAILLE8;
        assert_eq!(gs.fragment_to_glyph(0), "⠀");
        assert_eq!(gs.fragment_to_glyph(0b10_01_01_01), "⢇");
        assert_eq!(gs.fragment_to_glyph(0b11_11_11_11), "⣿");
    }
}
