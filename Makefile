.PHONY: clean default outdated print test

default:
	cargo build --release -p fac

clean:
	rm -rf Cargo.lock target/

outdated:
	cargo-outdated

print:
	git status --porcelain

test:
	cargo test --workspace
	# Ref: https://github.com/rust-lang/rust-clippy/issues/12270
	cargo clippy --workspace --tests --examples -- -A 'clippy::lint_groups_priority'
