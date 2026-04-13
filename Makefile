.PHONY: build install clean

build:
	cargo build --release

install: build
	install -m 755 target/release/lagent /usr/local/bin/lagent

clean:
	cargo clean
