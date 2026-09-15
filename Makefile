setup:
	cp .env.example .env
	docker compose up -d postgres

dev:
	cargo run

up:
	docker compose up --build

down:
	docker compose down

logs:
	docker compose logs -f

db:
	docker compose exec postgres psql -U app -d app

test:
	cargo test

clean:
	docker compose down -v

drop: 
