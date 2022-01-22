.PHONY: clean default test

default: target/release/fac

clean:
	rm -rf Cargo.lock target/

target/release/fac:
	cargo build --release -p fac

test:
	set -e; \
	for crate in 'factorio-mods-common' 'factorio-mods-local' 'factorio-mods-web' 'package' 'fac'; do \
		cargo test -p "$$crate"; \
		cargo clippy -p "$$crate"; \
		cargo clippy --tests -p "$$crate"; \
	done
