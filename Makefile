
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

