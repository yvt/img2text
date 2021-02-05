with import <nixpkgs> {};
with lib;

runCommand "dummy" rec {
  buildInputs = [
    wasm-pack
    wasm-bindgen-cli
    binaryen
    rustup
  ];
} ""
