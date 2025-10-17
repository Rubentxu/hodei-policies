# Hodei Tests

Tests para el framework Hodei y la aplicación de ejemplo.

## 📁 Estructura

```
tests/
├── app_example_tests.sh    # Tests para la aplicación de ejemplo (NUEVO)
├── api_tests.sh            # Tests para la API legacy
├── artifact_tests.sh       # Tests para artifacts
└── README.md               # Este archivo
```

## 🚀 Ejecutar Tests

### Opción 1: Script Automático (Recomendado)

El script `run_tests.sh` en la raíz del proyecto:
- Verifica servicios (PostgreSQL, Redis)
- Compila la aplicación
- Inicia el servidor
- Ejecuta los tests
- Limpia todo al finalizar

```bash
./run_tests.sh
```

### Opción 2: Manual

#### 1. Iniciar servicios

```bash
docker compose -f docker-compose.dev.yml up -d
```

#### 2. Configurar variables

```bash
export DATABASE_URL="postgres://postgres:postgres@localhost:5432/hodei"
export REDIS_URL="redis://localhost:6379"
```

#### 3. Iniciar servidor

```bash
cargo run -p app-example
```

#### 4. Ejecutar tests (en otra terminal)

```bash
bash tests/app_example_tests.sh
```

## 🧪 Tests de la Aplicación de Ejemplo

El archivo `app_example_tests.sh` prueba:

### Endpoints Básicos
- ✅ `GET /` - Info de la API
- ✅ `GET /health` - Health check

### Gestión de Usuarios
- ✅ `GET /users` - Listar usuarios
- ✅ `GET /users/:id` - Obtener usuario específico
- ✅ Manejo de usuarios no existentes

### Gestión de Documentos
- ✅ `GET /documents` - Listar documentos
- ✅ `GET /documents/:id` - Obtener documento
- ✅ `POST /documents` - Crear documento
- ✅ Validación de owner

### Autorización - Permisos de Owner
- ✅ Owner puede READ su documento
- ✅ Owner puede UPDATE su documento
- ✅ Owner puede DELETE su documento

### Autorización - No-Owner
- ✅ No-owner NO puede leer documentos privados
- ✅ No-owner NO puede actualizar documentos ajenos

### Autorización - Documentos Públicos
- ✅ Cualquiera puede leer documentos públicos
- ✅ No-owner NO puede actualizar documentos públicos

### Autorización - Roles (RBAC)
- ✅ Admin puede hacer TODO
- ✅ Editor puede leer y actualizar
- ✅ Viewer solo puede leer

## 📊 Ejemplo de Salida

```
🚀 Hodei Example Application - API Tests
📍 API URL: http://localhost:3000

🔍 Checking if server is running...
✓ Server is running

═══════════════════════════════════════════════════════════════
📋 BASIC ENDPOINTS
═══════════════════════════════════════════════════════════════
🧪 GET / - API Info ... ✓ PASS (HTTP 200)
🧪 GET /health - Health Check ... ✓ PASS (HTTP 200)

═══════════════════════════════════════════════════════════════
📋 USER ENDPOINTS
═══════════════════════════════════════════════════════════════
🧪 GET /users - List all users ... ✓ PASS (HTTP 200)
🧪 GET /users/:id - Get Alice ... ✓ PASS (HTTP 200)
...

═══════════════════════════════════════════════════════════════
📊 TEST SUMMARY
═══════════════════════════════════════════════════════════════
✓ Passed: 25
✗ Failed: 0
Total: 25

🎉 All tests passed!

✅ Validated Features:
   • Basic API endpoints
   • User management
   • Document CRUD
   • Owner-based permissions
   • Public document access
   • Role-based access control (RBAC)
   • Cedar Policy authorization
```

## 🔧 Requisitos

- **curl**: Para hacer requests HTTP
- **jq**: Para parsear JSON (opcional pero recomendado)
- **PostgreSQL**: Puerto 5432
- **Redis**: Puerto 6379
- **Rust**: 1.70+

### Instalar dependencias (Ubuntu/Debian)

```bash
sudo apt-get install curl jq postgresql-client redis-tools
```

### Instalar dependencias (macOS)

```bash
brew install curl jq postgresql redis
```

## 🐛 Troubleshooting

### Tests fallan con "Server is not running"

Asegúrate de que el servidor esté corriendo:
```bash
cargo run -p app-example
```

### Tests fallan con "Connection refused"

Verifica que los servicios estén corriendo:
```bash
docker compose -f docker-compose.dev.yml ps
pg_isready -h localhost -p 5432
redis-cli -h localhost -p 6379 ping
```

### Tests fallan con errores de autorización

Verifica que las políticas se cargaron correctamente en el log del servidor.

## 📝 Agregar Nuevos Tests

Para agregar un nuevo test, usa la función `run_test`:

```bash
run_test "Descripción del test" EXPECTED_HTTP_CODE \
    -X METHOD "$API_URL/endpoint" \
    -H "Header: value" \
    -d '{"json": "data"}'
```

Ejemplo:
```bash
run_test "Create new user" 201 \
    -X POST "$API_URL/users" \
    -H "Content-Type: application/json" \
    -d '{"email": "test@example.com", "name": "Test User"}'
```

## 📚 Referencias

- [Hodei Framework Documentation](../README.md)
- [Example Application](../crates/app-example/README.md)
- [Cedar Policy Language](https://docs.cedarpolicy.com/)

## 📄 License

MIT OR Apache-2.0
