.PHONY: check

check:
	cd api; cargo test --no-fail-fast -- --test-threads=1
	cd core; cargo test --no-fail-fast -- --test-threads=1

check-core:
	cd core; cargo test --no-fail-fast -- --test-threads=1
	cd api; cargo test --no-fail-fast -- --test-threads=1
