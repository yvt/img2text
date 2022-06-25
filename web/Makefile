
default: all
.PHONY: all wasm

all: wasm static/app.css static/FairfaxHD-subfont.ttf static/license.html static/index.html static/help.html

wasm:
	wasm-pack build --target no-modules --out-name wasm --out-dir ./static --no-typescript || exit $?
	rm static/package.json static/.gitignore

static/index.html: src/index.html
	cp src/index.html static/
static/help.html: src/help.html
	cp src/help.html static/

static/app.css: src/lib.less src/imagewell.less src/helpview.less
	lessc src/lib.less static/app.css || exit $?

static/FairfaxHD-subfont.ttf: vendor/fairfaxhd/FairfaxHD.ttf.gz src/font-subset.txt
	gunzip < vendor/fairfaxhd/FairfaxHD.ttf.gz > static/FairfaxHD.ttf
	pyftsubset static/FairfaxHD.ttf --output-file=static/FairfaxHD-subfont.woff2 \
		--flavor=woff2 --text-file=src/font-subset.txt
	rm -f static/FairfaxHD.ttf

static/license.html: ../Cargo.lock license-template.html
	cargo-license -j \
		| jq 'map(. + {authors: (.authors + "")})' \
		> static/license.json
	extrude-licenses --template-file license-template.html --input-file static/license.json \
		--escape-authors > static/license.html
	# It's probably not nice to put everyone's e-mail address on a public website
	sed -i.bak -e 's/ &lt;[^&]*&gt;//g' static/license.html
	rm static/license.html.bak
	# Replace the separators
	sed -i.bak -e 's/ *| */, /g' static/license.html
	rm static/license.html.bak
