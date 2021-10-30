.PHONY: clean default test

default: target/release/fac

clean:
	rm -rf Cargo.lock target/

target/release/fac:
	cargo build --release -p fac

test:
	cargo test --all
	cargo clippy --all
