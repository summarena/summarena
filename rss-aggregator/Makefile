.PHONY: postgres postgres-stop postgres-clean help

# Default target
help:
	@echo "Available commands:"
	@echo "  make postgres       - Start PostgreSQL container at localhost:5432"
	@echo "  make postgres-stop  - Stop PostgreSQL container"
	@echo "  make postgres-clean - Stop and remove PostgreSQL container and volume"

# Start PostgreSQL container
postgres:
	@echo "Starting PostgreSQL container..."
	@docker run -d \
		--name rss-postgres \
		-e POSTGRES_DB=rss_aggregator \
		-e POSTGRES_USER=rss_user \
		-e POSTGRES_PASSWORD=rss_password \
		-p 5432:5432 \
		-v rss_postgres_data:/var/lib/postgresql/data \
		postgres:15-alpine || echo "Container may already be running"
	@echo "PostgreSQL is starting up at localhost:5432"
	@echo "Database: rss_aggregator"
	@echo "User: rss_user"
	@echo "Password: rss_password"
	@echo "Connection string: postgresql://rss_user:rss_password@localhost:5432/rss_aggregator"
	@echo "Waiting for database to be ready..."
	@sleep 3
	@docker exec rss-postgres pg_isready -U rss_user -d rss_aggregator || echo "Database is starting up, please wait a few more seconds"

# Stop PostgreSQL container
postgres-stop:
	@echo "Stopping PostgreSQL container..."
	@docker stop rss-postgres || echo "Container may not be running"

# Clean up PostgreSQL container and data
postgres-clean:
	@echo "Stopping and removing PostgreSQL container and data..."
	@docker stop rss-postgres 2>/dev/null || true
	@docker rm rss-postgres 2>/dev/null || true
	@docker volume rm rss_postgres_data 2>/dev/null || true
	@echo "PostgreSQL cleanup complete"