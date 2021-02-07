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

      cargoSha256 = "10hz6094viyjlkdvhchc41a3m96hlknri8chh9s4nc93h582ps27";
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
    python37Packages.fonttools
    python37Packages.brotli
  ];
} ""
