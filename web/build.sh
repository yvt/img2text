#!/bin/sh
wasm-pack build --target no-modules --out-name wasm --out-dir ./static --no-typescript "$@" || exit $?
lessc src/lib.less static/app.css || exit $?
rm static/package.json static/.gitignore
