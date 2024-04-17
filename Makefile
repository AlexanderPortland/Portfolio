.PHONY: check

check:
	cd core; cargo test --no-fail-fast -- --test-threads=1
	cd api; cargo test --no-fail-fast -- --test-threads=1

