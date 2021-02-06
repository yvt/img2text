
default: all
.PHONY: all wasm

all: wasm static/app.css

wasm:
	wasm-pack build --target no-modules --out-name wasm --out-dir ./static --no-typescript || exit $?
	rm static/package.json static/.gitignore

static/app.css: src/lib.less src/imagewell.less
	lessc src/lib.less static/app.css || exit $?
