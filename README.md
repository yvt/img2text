```
img2text

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

USAGE:
    img2text [OPTIONS] <FILE>

ARGS:
    <FILE>
            The image to process

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
    -w <cell-width>
            The width of output characters, only used when `-s` is given without
            `!` [default: 0.45]

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
            The glyph set to use [default: slc] [possible values: slc, ms2x3,
            1x1, 1x2, 2x2, 2x3, braille]
```
