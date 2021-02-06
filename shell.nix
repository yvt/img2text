with import <nixpkgs> {};
with lib;

runCommand "dummy" rec {
  buildInputs = [
    wasm-pack
    wasm-bindgen-cli
    binaryen
    rustup
    lessc
    python37Packages.fonttools
    python37Packages.brotli
  ];
} ""
