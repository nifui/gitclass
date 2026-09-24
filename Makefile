
.PHONY: clean up down db test migrate reset
dev:
	cargo run --release

up:
	docker compose up -d

down:
	docker compose down

db:
	docker compose exec postgres psql -U app -d app

test:
	cargo test

clean:
	docker compose down -v

migrate: 
	cd backend && sqlx migrate run

reset: clean up migrate
