tests:
	cargo test -- --nocapture --test-threads=1

fmt:
	cargo +nightly fmt

fmt-dry:
	cargo +nightly fmt -- --emit stdout
