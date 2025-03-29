
test:
	cargo test --all
	echo Test: OK
	cargo test --all --release
	echo Test release: OK

lint:
	cargo clippy --all \
      -- \
      \
      -W clippy::all \
      -W clippy::pedantic \
      \
      -A clippy::missing_panics_doc \
      -A clippy::must_use_candidate \
      -A clippy::missing_errors_doc \
      -A clippy::module_name_repetitions \
      -A clippy::implicit_hasher \
      -A clippy::needless_pass_by_value \
      -A clippy::return_self_not_must_use \
      -A clippy::module_inception \
      -A clippy::manual_assert \
      \
      -D warnings

fmt:
	cargo +nightly fmt --all

pr:
	gh pr create --fill
