.PHONY: help build run test clean docker-up docker-down docker-logs migrate seed

help: ## Mostrar esta ayuda
	@echo "Comandos disponibles:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Compilar la aplicación
	cargo build --release --features schema-discovery

run: ## Ejecutar la aplicación localmente
	cargo run

test: ## Ejecutar tests de API
	@echo "🧪 Ejecutando tests de API..."
	@./tests/api_tests.sh

clean: ## Limpiar archivos de compilación
	cargo clean
	rm -rf target/

docker-up: ## Levantar servicios con Docker Compose
	@echo "🐳 Levantando servicios..."
	docker compose up -d
	@echo "⏳ Esperando a que los servicios estén listos..."
	@sleep 5
	@echo "✅ Servicios listos!"

docker-down: ## Detener servicios Docker
	docker compose down

docker-logs: ## Ver logs de los servicios
	docker compose logs -f

docker-rebuild: ## Reconstruir y levantar servicios
	docker compose down
	docker compose build --no-cache
	docker compose up -d

migrate: ## Ejecutar migraciones de base de datos
	@echo "📦 Instalando sqlx-cli si no está instalado..."
	@cargo install sqlx-cli --no-default-features --features postgres 2>/dev/null || true
	@echo "🔄 Ejecutando migraciones..."
	cargo sqlx database create
	cargo sqlx migrate run
	@echo "✅ Migraciones completadas!"

seed: ## Poblar base de datos con datos de prueba (se hace automáticamente al iniciar)
	@echo "ℹ️  El seed se ejecuta automáticamente al iniciar la aplicación"

dev: migrate ## Configurar entorno de desarrollo
	@echo "🔧 Configurando entorno de desarrollo..."
	@cp .env.example .env 2>/dev/null || true
	@echo "✅ Entorno configurado!"

dev-up: ## Levantar solo BD para desarrollo (sin app container)
	@echo "🐳 Levantando PostgreSQL + herramientas de BD..."
	docker compose -f docker-compose.dev.yml up -d
	@echo "⏳ Esperando a que PostgreSQL esté listo..."
	@sleep 5
	@echo "✅ Servicios listos!"
	@echo ""
	@echo "📊 Servicios disponibles:"
	@echo "   PostgreSQL: localhost:5432"
	@echo "   Adminer:    http://localhost:8080"
	@echo "   pgAdmin:    http://localhost:5050"
	@echo ""
	@echo "🚀 Para ejecutar la app:"
	@echo "   cargo run"

dev-down: ## Detener servicios de desarrollo
	docker compose -f docker-compose.dev.yml down

dev-logs: ## Ver logs de PostgreSQL en desarrollo
	docker compose -f docker-compose.dev.yml logs -f postgres

dev-restart: ## Reiniciar servicios de desarrollo
	docker compose -f docker-compose.dev.yml restart

dev-clean: ## Limpiar datos de desarrollo (¡CUIDADO! Borra la BD)
	docker compose -f docker-compose.dev.yml down -v
	@echo "⚠️  Datos de desarrollo eliminados"

docker-test: docker-up ## Ejecutar tests contra Docker
	@echo "⏳ Esperando a que la aplicación esté lista..."
	@sleep 10
	@echo "🧪 Ejecutando tests..."
	@API_URL=http://localhost:3000 ./tests/api_tests.sh
	@echo ""
	@echo "🛑 Deteniendo servicios..."
	@docker compose down

all: build docker-up docker-test ## Compilar, levantar y probar todo

status: ## Ver estado de los servicios
	@docker compose ps

logs-app: ## Ver logs de la aplicación
	@docker compose logs -f app

logs-db: ## Ver logs de PostgreSQL
	@docker compose logs -f postgres

shell-app: ## Abrir shell en el contenedor de la aplicación
	@docker compose exec app /bin/bash

shell-db: ## Abrir psql en PostgreSQL
	@docker compose exec postgres psql -U postgres -d hodei_policies

adminer: ## Abrir Adminer en el navegador
	@echo "🌐 Abriendo Adminer en http://localhost:8080"
	@echo "   Sistema: PostgreSQL"
	@echo "   Servidor: postgres"
	@echo "   Usuario: postgres"
	@echo "   Contraseña: postgres"
	@echo "   Base de datos: hodei_policies"
	@xdg-open http://localhost:8080 2>/dev/null || open http://localhost:8080 2>/dev/null || echo "   Abre manualmente: http://localhost:8080"

pgadmin: ## Abrir pgAdmin en el navegador
	@echo "🌐 Abriendo pgAdmin en http://localhost:5050"
	@echo "   Email: admin@hodei.com"
	@echo "   Contraseña: admin"
	@xdg-open http://localhost:5050 2>/dev/null || open http://localhost:5050 2>/dev/null || echo "   Abre manualmente: http://localhost:5050"

backup-db: ## Crear backup de la base de datos
	@echo "💾 Creando backup..."
	@docker compose exec postgres pg_dump -U postgres hodei_policies > backup_$(shell date +%Y%m%d_%H%M%S).sql
	@echo "✅ Backup creado: backup_$(shell date +%Y%m%d_%H%M%S).sql"

restore-db: ## Restaurar backup (uso: make restore-db FILE=backup.sql)
	@if [ -z "$(FILE)" ]; then \
		echo "❌ Error: Especifica el archivo con FILE=backup.sql"; \
		exit 1; \
	fi
	@echo "📥 Restaurando backup desde $(FILE)..."
	@docker compose exec -T postgres psql -U postgres hodei_policies < $(FILE)
	@echo "✅ Backup restaurado!"

check: ## Verificar código sin compilar
	cargo check

fmt: ## Formatear código
	cargo fmt

clippy: ## Ejecutar linter
	cargo clippy -- -D warnings

doc: ## Generar documentación
	cargo doc --no-deps --open

schema: ## Regenerar esquema Cedar
	cargo build --features schema-discovery
	@echo "✅ Esquema generado en cedar_schema.json"
	@cat cedar_schema.json | jq .
