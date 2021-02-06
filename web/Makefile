
default: all
.PHONY: all wasm

all: wasm static/app.css static/FairfaxHD-subfont.ttf

wasm:
	wasm-pack build --target no-modules --out-name wasm --out-dir ./static --no-typescript || exit $?
	rm static/package.json static/.gitignore

static/app.css: src/lib.less src/imagewell.less
	lessc src/lib.less static/app.css || exit $?

static/FairfaxHD-subfont.ttf: vendor/fairfaxhd/FairfaxHD.ttf.gz src/font-subset.txt
	gunzip < vendor/fairfaxhd/FairfaxHD.ttf.gz > static/FairfaxHD.ttf
	pyftsubset static/FairfaxHD.ttf --output-file=static/FairfaxHD-subfont.woff2 \
		--flavor=woff2 --text-file=src/font-subset.txt
	rm -f static/FairfaxHD.ttf
