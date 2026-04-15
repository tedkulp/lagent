default:
    just --list

test:
    cargo test

build:
    cargo build --release

install:
    cargo install --path .

clean:
    cargo clean
