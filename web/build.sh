#!/bin/sh
wasm-pack build --target no-modules --out-name wasm --out-dir ./static --no-typescript "$@" || exit $?
rm static/package.json static/.gitignore
