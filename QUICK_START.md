# 🚀 Quick Start - Hodei Example Application

Guía rápida para ejecutar y probar la aplicación de ejemplo.

## 📋 Requisitos

- Docker
- Rust 1.70+
- curl (para tests)
- jq (opcional, para ver JSON formateado)

## ⚡ Inicio Rápido (3 pasos)

### 1️⃣ Iniciar Servicios

```bash
docker compose -f docker-compose.dev.yml up -d
```

Esto inicia:
- **PostgreSQL** en `localhost:5432`
- **Redis** en `localhost:6379`
- **Adminer** (UI PostgreSQL) en `http://localhost:8080`
- **pgAdmin** (UI PostgreSQL) en `http://localhost:5050`

### 2️⃣ Ejecutar la Aplicación

```bash
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/hodei_policies"
export REDIS_URL="redis://localhost:6379"

cargo run -p app-example
```

Deberías ver:
```
🚀 Starting Hodei Example Application
✅ Connected to PostgreSQL
✅ Authorization service initialized
✅ Sample data created: 3 users, 3 documents
🌐 Server listening on http://0.0.0.0:3000
```

### 3️⃣ Probar la API

En otra terminal:

```bash
# Ver usuarios
curl http://localhost:3000/users | jq

# Ver documentos
curl http://localhost:3000/documents | jq

# Probar autorización
curl -X POST http://localhost:3000/documents/doc-1/check \
  -H "Content-Type: application/json" \
  -d '{
    "user_email": "alice@example.com",
    "action": "DocApp::Action::\"Document::Read\""
  }' | jq
```

## 🧪 Ejecutar Tests Automáticos

```bash
# Opción 1: Script automático (inicia servidor y ejecuta tests)
./run_tests.sh

# Opción 2: Solo tests (servidor debe estar corriendo)
bash tests/app_example_tests.sh
```

## 📊 Endpoints Disponibles

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/` | Info de la API |
| GET | `/health` | Health check |
| GET | `/users` | Listar usuarios |
| GET | `/users/:id` | Obtener usuario |
| GET | `/documents` | Listar documentos |
| GET | `/documents/:id` | Obtener documento |
| POST | `/documents` | Crear documento |
| POST | `/documents/:id/check` | **Verificar autorización** ⭐ |

## 👥 Usuarios de Ejemplo

La aplicación crea automáticamente 3 usuarios:

| Usuario | Email | Rol | Permisos |
|---------|-------|-----|----------|
| **Alice** | alice@example.com | Admin | Puede hacer TODO |
| **Bob** | bob@example.com | Editor | Puede leer y actualizar |
| **Charlie** | charlie@example.com | Viewer | Solo puede leer |

## 📄 Documentos de Ejemplo

| Documento | Owner | Público | Descripción |
|-----------|-------|---------|-------------|
| Alice's Private | Alice | ❌ No | Solo Alice puede acceder |
| Bob's Public | Bob | ✅ Sí | Todos pueden leer |
| Shared Policy | Alice | ✅ Sí | Documento público de Alice |

## 🎯 Ejemplos de Autorización

### ✅ ALLOW - Alice lee su documento privado

```bash
curl -X POST http://localhost:3000/documents/doc-1/check \
  -H "Content-Type: application/json" \
  -d '{
    "user_email": "alice@example.com",
    "action": "DocApp::Action::\"Document::Read\""
  }'
```

**Resultado**: `"authorized": true, "decision": "ALLOW"`

### ❌ DENY - Charlie intenta actualizar documento de Alice

```bash
curl -X POST http://localhost:3000/documents/doc-1/check \
  -H "Content-Type: application/json" \
  -d '{
    "user_email": "charlie@example.com",
    "action": "DocApp::Action::\"Document::Update\""
  }'
```

**Resultado**: `"authorized": false, "decision": "DENY"`

### ✅ ALLOW - Cualquiera lee documento público

```bash
curl -X POST http://localhost:3000/documents/doc-2/check \
  -H "Content-Type: application/json" \
  -d '{
    "user_email": "charlie@example.com",
    "action": "DocApp::Action::\"Document::Read\""
  }'
```

**Resultado**: `"authorized": true, "decision": "ALLOW"` (porque es público)

## 🔧 Comandos Útiles

### Ver logs del servidor
```bash
# Si usas el script run_tests.sh
tail -f /tmp/hodei-server.log
```

### Verificar servicios
```bash
# PostgreSQL
pg_isready -h localhost -p 5432

# Redis
redis-cli -h localhost -p 6379 ping

# Servidor
curl http://localhost:3000/health
```

### Detener servicios
```bash
docker compose -f docker-compose.dev.yml down
```

### Reiniciar base de datos
```bash
docker compose -f docker-compose.dev.yml down -v  # Elimina volúmenes
docker compose -f docker-compose.dev.yml up -d
```

### Ver logs
```bash
docker compose -f docker-compose.dev.yml logs -f postgres
docker compose -f docker-compose.dev.yml logs -f redis
```

## 📚 Más Información

- [README Principal](README.md)
- [Documentación de Tests](tests/README.md)
- [Aplicación de Ejemplo](crates/app-example/README.md)
- [Guía de Publicación](PUBLICATION_GUIDE.md)

## 🐛 Troubleshooting

### Error: "Connection refused" (PostgreSQL)

```bash
# Verificar que PostgreSQL está corriendo
docker compose ps

# Ver logs
docker compose logs postgres

# Reiniciar
docker compose restart postgres
```

### Error: "Connection refused" (Redis)

```bash
# Verificar que Redis está corriendo
docker compose ps

# Ver logs
docker compose logs redis

# Reiniciar
docker compose restart redis
```

### Error: "Server failed to start"

```bash
# Ver logs del servidor
cat /tmp/hodei-server.log

# Verificar que el puerto 3000 está libre
lsof -i :3000

# Matar proceso si es necesario
kill $(lsof -t -i:3000)
```

## ✅ Checklist

- [ ] Docker instalado y corriendo
- [ ] Servicios iniciados (`docker compose up -d`)
- [ ] Variables de entorno configuradas
- [ ] Aplicación compilando (`cargo build -p app-example`)
- [ ] Servidor corriendo (`cargo run -p app-example`)
- [ ] Tests pasando (`bash tests/app_example_tests.sh`)

---

**¡Listo para usar!** 🎉

Si todo funciona, deberías ver todos los tests en verde ✅
