.PHONY: coverage test-coverage-profraw

test-coverage-profraw:
	CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='coverage/cargo-test-%p-%m.profraw' cargo test

# this recuires `rustup component add llvm-tools-preview` and `cargo install grcov`
coverage/lcov.info: $(wildcard **/coverage/*.profraw)
	grcov . --binary-path ./target/debug/deps/ -s . -t lcov --branch --ignore-not-existing -o coverage/lcov.info

coverage: coverage/lcov.info

