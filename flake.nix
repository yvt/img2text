{
  description = "Image-to-text converter";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-22.05";
    flake-utils.url = "github:numtide/flake-utils";
  };
  
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        extrude-licenses =
          pkgs.rustPlatform.buildRustPackage rec {
            pname = "extrude-licenses";
            version = "1.0.0";

            src = pkgs.fetchFromGitHub {
              owner = "usagi";
              repo = pname;
              rev = version;
              sha256 = "1sg64rbv165zmigmjf7xrrlpqibhl13zvlqzkx1n3ribfvhy5lh0";
            };

            cargoSha256 = "kkPQqwxFpez3ffCx0NtL8wipkVTs9tRiLd6T0/6yTJ8=";
          };
      in
        {
          packages.default = pkgs.mkShell {
            # TODO: support building the img2text CLI and web app by flake
            nativeBuildInputs =
              (with pkgs;
                [
                  pkg-config
                  cargo-license
                  extrude-licenses
                  wasm-pack
                  wasm-bindgen-cli
                  binaryen
                  rustup
                  lessc
                  openssl_1_1
                  libiconv
                  python3Packages.fonttools
                  python3Packages.brotli
                ])
              ++ [ extrude-licenses ];
          };
        });
}
