.PHONY: test

test:
	CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUNNER='firetest' cargo test --target='x86_64-unknown-linux-musl'
