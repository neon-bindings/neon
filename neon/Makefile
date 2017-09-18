MARKDOWN_FILES = $(wildcard doc/*.md)
HTML_FILES = $(MARKDOWN_FILES:doc/%.md=target/doc/%.html)

all: doc

publish: doc
	cd target/doc && surge

doc: neon_api neon_build_api neon_runtime_api $(HTML_FILES) target/doc/rust.css target/doc/CNAME

clean:
	rm -rf target/doc

neon_api:
	cargo doc

neon_build_api:
	cd crates/neon_build && cargo doc
	cp -R crates/neon-build/target/doc/neon_build ./target/doc

neon_runtime_api:
	cd crates/neon-runtime && cargo doc
	cp -R crates/neon-runtime/target/doc/neon_runtime ./target/doc/

target/doc/%.html: doc/%.md
	rustdoc --markdown-playground-url='https://play.rust-lang.org' --markdown-css rust.css $< --output=target/doc

target/doc/rust.css:
	curl -s https://raw.githubusercontent.com/rust-lang/rust/master/src/doc/rust.css > target/doc/rust.css

target/doc/CNAME: doc/CNAME
	cp doc/CNAME target/doc/CNAME
