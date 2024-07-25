
test:
	cargo test --all
	echo Test: OK
	cargo test --all --release
	echo Test release: OK

lint:
	cargo clippy \
      -- \
      \
      -W clippy::all \
      -W clippy::pedantic \
      \
#      -A clippy::missing_panics_doc \
      \
      -D warnings

fmt:
	cargo +nightly fmt --all
