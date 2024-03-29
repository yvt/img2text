# `img2text`

<a href="https://crates.io/crates/img2text"><img src="https://img.shields.io/crates/v/img2text?style=for-the-badge"></a>

## `img2text` on Terminal

```
img2text

Image-to-text converter

                 ⠀⠀⠀⠀⠀⠀⠀⢠⣄⣠⣶⣤⣿⣤⣶⣄⣠⡄⠀⠀⠀⠀⠀⠀⠀
                 ⠀⠀⠀⠀⣄⣸⣿⣾⡿⠿⠛⢿⣀⡿⠛⠿⢿⣷⣿⣇⣠⠀⠀⠀⠀
                 ⠀⠀⢠⣤⣿⣿⣛⣁⣀⣀⣀⣀⣉⣀⣀⣀⡀⠈⠛⢿⣿⣤⡄⠀⠀
                 ⠀⠲⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡄⠈⣿⣿⣷⠖⠀
                 ⠐⢾⣿⣤⠼⠇⢸⣿⣿⣿⣇⣀⣀⣀⣹⣿⣿⣿⠇⠸⢧⣤⣿⡷⠂
    powered by:  ⠰⢿⣿⡇⠀⠀⢸⣿⣿⣿⡿⠿⠿⢿⣿⣿⣿⣦⠀⠀⢀⣸⣿⡿⠆
                 ⠐⠿⣿⣷⣤⣤⣼⣿⣿⣿⣧⣤⣄⠀⢻⣿⣿⣿⣦⣤⣾⣿⣿⠿⠂
                 ⠀⠚⢿⣿⣿⡿⠿⠿⠿⠿⠿⠿⠿⠀⠈⠻⠿⠿⠿⢿⣿⣿⡿⠓⠀
                 ⠀⠀⠘⠛⣿⣿⡟⢻⡆⠀⠀⠀⠀⠀⠀⠀⣸⠛⢳⣿⣿⠛⠃⠀⠀
                 ⠀⠀⠀⠀⠉⢹⡿⢿⣿⣷⣶⣶⣶⣶⣶⣾⣿⡿⢿⡏⠉⠀⠀⠀⠀
                 ⠀⠀⠀⠀⠀⠀⠀⠈⠉⠘⠟⠙⠿⠋⠻⠃⠉⠁⠀⠀⠀⠀⠀⠀⠀

(The above image was generated by this program with an option `-s 25`.)

USAGE:
    img2text [OPTIONS] <FILE>

ARGS:
    <FILE>
            The image to process

FLAGS:
    -d, --dither
            Apply dithering to preserve the gray shades. Incompatible with `-i
            edge-canny`

    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
    -w <cell-width>
            The width of output characters, only used when `-s` is given without
            `!` [default: 0.45]

        --dither-contrast <dither-contrast>
            Choose the contrast enhancing technique to use for dithering
            [default: median-quant] [possible values: none, median-quant,
            equalize]

        --canny-high-threshold <edge-canny-high-threshold>
            A parameter for the Canny edge detector (`-i edge-canny`).

            Edges with a strength higher than the high threshold will always
            appear as edges in the output image. [default: 20]

        --canny-low-threshold <edge-canny-low-threshold>
            A parameter for the Canny edge detector (`-i edge-canny`).

            Edges with a strength higher than the low threshold will appear in
            the output image if there are strong edges nearby. [default: 10]

    -i <input-ty>
            Specifies how to interpret the input image [default: auto] [possible
            values: auto, wob, bow, edge-canny]

    -s <out-size>
            The output size, measured in character cells or percents (e.g.,
            `80`, `80x40`, `80x40!`, `-80x40`, `100%`). [default: downscale to
            terminal size (if the output is a terminal) or 100% (otherwise)]

            - 80: Fit within 80x80 character cells

            - 80x40: Fit within 80x40 character cells, upscaling as necessary

            - -80x40: Fit within 80x40 character cells, only downscaling

            - 80x40!: Fit to 80x40 character cells, not maintaining the aspect
            ratio

            - 150%: Scale by 150%. The actual output size depends on the glyph
            set being used; for example, `2x3` maps each 2x3 block to one
            character.

    -g <style>
            The glyph set to use [default: braille] [possible values: slc, 
            ms2x3, 1x1, 1x2, 2x2, 2x3, braille]
```

### Installing

First, make sure [rustup](https://www.rust-lang.org/tools/install) or Rust 1.49.0 or later is installed. Then run the following command:

```
cargo install img2text
```

This will compile and install `img2text` the CLI app to `~/.cargo/bin` or somewhere else in your system.

### Recommended Font

[Fairfax HD](http://www.kreativekorp.com/software/fonts/fairfaxhd.shtml) can display all characters (particularly [Symbols for Legacy Computing]) generated by this program.

[Symbols for Legacy Computing]: https://en.wikipedia.org/wiki/Symbols_for_Legacy_Computing

## `img2text` on Web

<https://img2text.yvt.jp> is a single-page static website.

### Developing

Prerequisites:

 - [`wasm-pack`](https://crates.io/crates/wasm-pack)
 - [`cargo-license`](https://crates.io/crates/cargo-license)
 - [`extrude-licenses`](https://crates.io/crates/extrude-licenses)
 - [Binaryen](https://github.com/WebAssembly/binaryen)
 - [rustup](https://www.rust-lang.org/tools/install)
 - [lessc](http://lesscss.org)
 - [FontTools](https://github.com/fonttools/fonttools)
 - ... or just use [Nix](https://nixos.org) and run `nix develop` to install all of them
 - (*Optional*) [`cargo-watch`](https://crates.io/crates/cargo-watch)

```shell
cd web
make
python -m http.server
```

To continuously rebuild:

```shell
cd web/static
python -m http.server &
cd ..
cargo watch -s make -i static
```

## `img2text` in Your App

Add the following to your app's `Cargo.toml` file:

```toml
[dependencies]
img2text = { version = "0.1.0", default-features = false }
```
