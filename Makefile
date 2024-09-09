.PHONY: check api check-api test frontend backend back front

check-api:
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/ && \
	cd api; cargo test -- --test-threads=1

check: check-api
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/ && \
	cd core; cargo test -- --test-threads=1
	
backend:
	export PORTFOLIO_DATABASE_URL=mysql://root:@127.0.0.1/; \
	cargo run

frontend:
	rustup default nightly-2022-09-24
	export PORTFOLIO_API_HOST=127.0.0.1:8000 && \
	cd frontend && npm install && npm run dev

test: check
back: backend
front: frontend

api: check-api