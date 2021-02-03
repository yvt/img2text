use std::{env, fmt::Write, fs, path::Path};

#[path = "src/int.rs"]
mod int;
use self::int::BinInteger as _;

fn main() {
    let mut glyphsets_rs = String::new();

    for &glyph_set_in in GLYPH_SETS {
        process_one_glyph_set(&mut glyphsets_rs, glyph_set_in);
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("glyphsets.rs");
    fs::write(&dest_path, glyphsets_rs).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}

fn process_one_glyph_set(glyphsets_rs: &mut String, gs: &GlyphSetIn) {
    macro_rules! wl {
        ($($tt:tt)*) => { writeln!(glyphsets_rs, $($tt)*).unwrap() };
    }

    let mask_dims = gs.mask_dims;

    #[derive(Clone, Copy)]
    struct IndexEnt {
        glyph_i: usize,
        distance: usize,
    }
    let mut index = vec![None; 1 << (mask_dims[0] * mask_dims[1])];

    // Put the exact matches
    for (glyph_i, &(_, mask)) in gs.glyphs.iter().enumerate() {
        let mask = decode_mask(mask, mask_dims);
        index[mask as usize] = Some(IndexEnt {
            glyph_i,
            distance: 0,
        });
    }

    // Mutate known patterns to create lesser matches
    let mut last_distance = 0;
    for base_distance in 0.. {
        let mut should_continue = false;

        for mask in 0..index.len() {
            if let Some(IndexEnt { distance, glyph_i }) = index[mask] {
                if distance != base_distance {
                    // Already processed by a previous iteration
                    continue;
                }

                mutate_fragment_by_dilation_and_erosion(mask_dims, mask as Fragment, |new_mask| {
                    if index[new_mask as usize].is_some() {
                        return;
                    }

                    index[new_mask as usize] = Some(IndexEnt {
                        glyph_i,
                        distance: base_distance + 1,
                    });
                    eprintln!("{:09b} -> {:09b}, {}", mask, new_mask, base_distance + 1);
                    should_continue = true;
                });
            }
        }

        if !should_continue {
            break;
        }

        last_distance = base_distance;
    }

    // The above mutation technique doesn't cover the entire space. Now bring a
    // bigger gun
    for base_distance in 0.. {
        let mut should_continue = false;

        for mask in 0..index.len() {
            if let Some(IndexEnt { distance, glyph_i }) = index[mask] {
                if distance != base_distance {
                    // Already processed by a previous iteration
                    continue;
                }

                mutate_fragment_unconditional(mask_dims, mask as Fragment, |new_mask| {
                    if index[new_mask as usize].is_some() {
                        return;
                    }

                    index[new_mask as usize] = Some(IndexEnt {
                        glyph_i,
                        distance: base_distance + 1,
                    });
                    eprintln!("{:09b} -> {:09b}, {}", mask, new_mask, base_distance + 1);
                    should_continue = true;
                });
            }
        }

        if !should_continue && base_distance > last_distance {
            break;
        }
    }

    let max_glyph_len = gs.glyphs.iter().map(|x| x.0.len()).max().unwrap();

    wl!("pub const {}: &dyn GlyphSet =", gs.const_name);
    wl!("    &IndexedGlyphSet {{");
    wl!(
        "        mask_dims: [{}, {}],",
        gs.mask_dims[0],
        gs.mask_dims[1]
    );
    wl!("        max_glyph_len: {},", max_glyph_len);
    wl!("        index: &[");
    for ent in index.iter() {
        wl!(
            "            r##\"{}\"##,",
            gs.glyphs[ent.unwrap().glyph_i].0
        );
    }
    wl!("        ],");
    wl!("    }};");
}

/// Mutate a fragment.
fn mutate_fragment_by_dilation_and_erosion(
    [w, h]: [usize; 2],
    frag: Fragment,
    mut cb: impl FnMut(Fragment),
) {
    let mut i = 0;
    for y in 0..h {
        for x in 0..w {
            let cur = frag.get_bit(i);

            // Deny dilation (the outcome is opposite - the result image will
            // be dilated)
            if cur {
                continue;
            }

            // try copying this pixel to a neighboring one
            for &(sx, sy) in &[(-1isize, 0), (1, 0), (0, -1isize), (0, 1)] {
                let nx = x.wrapping_add(sx as usize);
                let ny = y.wrapping_add(sy as usize);
                if nx >= w || ny >= h {
                    continue; // OOB
                }

                let ni = (nx + ny * w) as u32;
                let nei = frag.get_bit(ni);

                if cur == nei {
                    continue; // no change
                }

                // mutate
                let new_frag = frag ^ (1 << ni);

                cb(new_frag);
            }

            i += 1;
        }
    }
}

/// Mutate a fragment differently.
fn mutate_fragment_unconditional([w, h]: [usize; 2], frag: Fragment, mut cb: impl FnMut(Fragment)) {
    for i in 0..w * h {
        cb(frag ^ (1 << i));
    }
}

const GLYPH_SETS: &[&GlyphSetIn] = &[&GS_DEFAULT];

/// A small bitmap image, whose dimensions are specified implciitly (e.g., by
/// `GlyphSetIn::mask_dims`).
type Fragment = u32;

struct GlyphSetIn {
    const_name: &'static str,
    mask_dims: [usize; 2],
    glyphs: &'static [(&'static str, Fragment)],
}

/// The masks in `GlyphSetIn` are in reverse order so that their writing
/// direction matches the visual order. This function converts a mask to
/// the standard order (LSB = upper left corner, MSB = lower right corner).
fn decode_mask(mask: Fragment, mask_dims: [usize; 2]) -> Fragment {
    mask.reverse_bits() >> (32 - (mask_dims[0] * mask_dims[1]))
}

const GS_DEFAULT: GlyphSetIn = GlyphSetIn {
    const_name: "GLYPH_SET_DEFAULT",
    mask_dims: [3, 3],
    glyphs: &[
        ("█", 0b111_111_111),
        ("🭪", 0b111_110_111),
        ("🭨", 0b111_011_111),
        ("🭖", 0b111_011_011),
        ("🭡", 0b111_110_110),
        ("🭐", 0b110_110_111),
        ("🭅", 0b011_011_111),
        ("🭒", 0b111_111_011),
        ("🭝", 0b111_111_110),
        ("🭌", 0b110_111_111),
        ("🭁", 0b011_111_111),
        ("🬎", 0b111_111_000),
        ("🬹", 0b000_111_111),
        ("▋", 0b110_110_110),
        ("🮉", 0b011_011_011),
        ("🮇", 0b001_001_001),
        ("🮂", 0b111_000_000),
        ("▍", 0b100_100_100),
        ("▂", 0b000_000_111),
        ("🭲", 0b010_010_010),
        ("🭹", 0b000_111_000),
        ("╋", 0b010_111_010),
        ("◢", 0b001_011_111),
        ("◥", 0b111_011_001),
        ("◢", 0b001_011_111),
        ("◥", 0b111_011_001),
        ("🬏", 0b000_000_110),
        ("🬾", 0b000_100_100),
        ("🭙", 0b100_100_000),
        ("🬀", 0b110_000_000),
        ("🬁", 0b011_000_000),
        ("🭤", 0b001_001_000),
        ("🭉", 0b000_001_001),
        ("🬞", 0b000_000_011),
        ("╺", 0b000_011_000),
        ("╹", 0b010_010_000),
        ("╸", 0b000_110_000),
        ("╻", 0b000_010_010),
        ("🭢", 0b001_000_000),
        ("🭗", 0b100_000_000),
        ("🬼", 0b000_000_100),
        ("🭇", 0b000_000_001),
        (" ", 0b000_000_000),
    ],
};
