with import <nixpkgs> {};
with lib;

let
  extrude-licenses =
    rustPlatform.buildRustPackage rec {
      pname = "extrude-licenses";
      version = "1.0.0";

      src = fetchFromGitHub {
        owner = "usagi";
        repo = pname;
        rev = version;
        sha256 = "1sg64rbv165zmigmjf7xrrlpqibhl13zvlqzkx1n3ribfvhy5lh0";
      };

      cargoSha256 = "kkPQqwxFpez3ffCx0NtL8wipkVTs9tRiLd6T0/6yTJ8=";
    };
in

runCommand "dummy" rec {
  buildInputs = [
    cargo-license
    extrude-licenses
    wasm-pack
    wasm-bindgen-cli  # TODO: somehow lock the version of `wasm-bindgen`
    binaryen
    rustup
    lessc
    python3Packages.fonttools
    python3Packages.brotli
  ];
} ""
