# Guía de Desarrollo Rápido

## 🚀 Desarrollo Local con Cargo

Esta guía te permite desarrollar rápidamente sin necesidad de reconstruir contenedores Docker.

---

## ⚡ Inicio Rápido (3 pasos)

```bash
# 1. Levantar solo PostgreSQL + herramientas
make dev-up

# 2. Ejecutar la aplicación con hot-reload
cargo run

# 3. En otra terminal, hacer cambios y recompilar automáticamente
cargo watch -x run
```

---

## 📋 Flujo de Trabajo Recomendado

### Primera Vez (Setup Inicial)

```bash
# 1. Copiar variables de entorno
cp .env.example .env

# 2. Levantar PostgreSQL
make dev-up

# 3. Ejecutar migraciones
make migrate

# 4. Ejecutar aplicación
cargo run
```

### Desarrollo Diario

```bash
# Terminal 1: PostgreSQL ya está corriendo
make dev-up  # Solo si no está corriendo

# Terminal 2: Ejecutar app con hot-reload
cargo watch -x run

# Terminal 3: Ejecutar tests
cargo test
# o
make test  # Tests de API
```

---

## 🛠️ Comandos de Desarrollo

### Gestión de Servicios

```bash
make dev-up        # Levantar PostgreSQL + Adminer + pgAdmin
make dev-down      # Detener servicios
make dev-restart   # Reiniciar servicios
make dev-logs      # Ver logs de PostgreSQL
make dev-clean     # Limpiar TODO (¡cuidado!)
```

### Desarrollo de la App

```bash
cargo run          # Ejecutar app
cargo check        # Verificar compilación rápida
cargo build        # Compilar
cargo test         # Ejecutar tests unitarios
cargo watch -x run # Auto-recompilar al guardar cambios
```

### Base de Datos

```bash
make migrate       # Ejecutar migraciones
make shell-db      # Abrir psql
make adminer       # Abrir Adminer en navegador
make pgadmin       # Abrir pgAdmin en navegador
make backup-db     # Crear backup
```

### Calidad de Código

```bash
make fmt           # Formatear código
make clippy        # Linter
make check         # Verificar sin compilar
cargo test         # Tests unitarios
```

---

## 🔧 Herramientas Recomendadas

### 1. cargo-watch (Auto-reload)

```bash
# Instalar
cargo install cargo-watch

# Usar
cargo watch -x run                    # Recompilar al guardar
cargo watch -x check                  # Solo verificar
cargo watch -x "run --features dev"   # Con features
```

### 2. rust-analyzer (IDE)

- **VSCode**: Instalar extensión "rust-analyzer"
- **IntelliJ**: Instalar plugin "Rust"

### 3. Adminer (Gestión de BD)

```bash
make adminer  # Abre http://localhost:8080
```

---

## 📊 Servicios Disponibles

| Servicio   | URL                    | Credenciales                    |
|------------|------------------------|---------------------------------|
| App        | http://localhost:3000  | N/A                             |
| PostgreSQL | localhost:5432         | postgres / postgres             |
| Adminer    | http://localhost:8080  | postgres / postgres / hodei_policies |
| pgAdmin    | http://localhost:5050  | admin@hodei.com / admin         |

---

## 🔄 Workflow Típico

### Agregar una Nueva Feature

```bash
# 1. Crear rama
git checkout -b feature/nueva-feature

# 2. Asegurar que PostgreSQL esté corriendo
make dev-up

# 3. Hacer cambios en el código
# ... editar archivos ...

# 4. Verificar compilación
cargo check

# 5. Ejecutar app
cargo run

# 6. Probar en otra terminal
curl http://localhost:3000/health

# 7. Ejecutar tests
cargo test
make test

# 8. Formatear y lint
make fmt
make clippy

# 9. Commit
git add .
git commit -m "feat: nueva feature"
```

### Modificar el Esquema de BD

```bash
# 1. Crear nueva migración
sqlx migrate add nombre_de_migracion

# 2. Editar archivo en migrations/
# ... editar SQL ...

# 3. Ejecutar migración
make migrate

# 4. Verificar en Adminer
make adminer

# 5. Actualizar modelos en código
# ... editar src/domain.rs o hodei_domain/src/lib.rs ...

# 6. Recompilar
cargo run
```

### Agregar Nueva Entidad

```bash
# 1. Editar hodei_domain/src/lib.rs
# Agregar struct con #[derive(HodeiEntity)]

# 2. Recompilar con schema-discovery
cargo build --features schema-discovery

# 3. Verificar esquema generado
cat cedar_schema.json | jq .

# 4. Agregar migración si es necesario
sqlx migrate add add_nueva_entidad_table

# 5. Ejecutar migración
make migrate

# 6. Probar
cargo run
```

---

## 🐛 Debugging

### Ver Logs Detallados

```bash
# Nivel de log detallado
RUST_LOG=debug cargo run

# Solo logs de la app
RUST_LOG=hodei_cedar_mvp_kernel=debug cargo run

# Logs de sqlx
RUST_LOG=sqlx=debug cargo run
```

### Debugging con rust-lldb

```bash
# Compilar con símbolos de debug
cargo build

# Ejecutar con debugger
rust-lldb target/debug/hodei_cedar_mvp_kernel
```

### Verificar Conexión a BD

```bash
# Desde la app
RUST_LOG=sqlx=debug cargo run

# Desde psql
make shell-db

# Desde Adminer
make adminer
```

---

## 🔥 Hot Reload Avanzado

### Opción 1: cargo-watch (Recomendado)

```bash
# Básico
cargo watch -x run

# Con clear de pantalla
cargo watch -c -x run

# Con tests antes de ejecutar
cargo watch -x test -x run

# Con check rápido
cargo watch -x check -x run
```

### Opción 2: systemfd + cargo-watch

```bash
# Instalar
cargo install systemfd cargo-watch

# Ejecutar con hot-reload de sockets
systemfd --no-pid -s http::3000 -- cargo watch -x run
```

---

## 📈 Optimización de Compilación

### Acelerar Compilaciones

```toml
# En .cargo/config.toml (crear si no existe)
[build]
incremental = true

[profile.dev]
opt-level = 0
debug = true

[profile.dev.package."*"]
opt-level = 3  # Optimizar dependencias
```

### Usar mold (Linker rápido)

```bash
# Instalar mold
sudo apt install mold  # Ubuntu/Debian

# Usar en compilación
mold -run cargo build
```

---

## 🧪 Testing en Desarrollo

### Tests Unitarios

```bash
# Todos los tests
cargo test

# Tests específicos
cargo test test_name

# Con output
cargo test -- --nocapture

# Tests de un módulo
cargo test auth::
```

### Tests de Integración

```bash
# Asegurar que PostgreSQL esté corriendo
make dev-up

# Ejecutar tests de API
make test

# Test individual
curl -X POST http://localhost:3000/documents \
  -H "Authorization: Bearer alice" \
  -H "Content-Type: application/json" \
  -d '{"resource_id":"test","is_public":false}'
```

---

## 💡 Tips y Trucos

### 1. Alias Útiles

```bash
# Agregar a ~/.bashrc o ~/.zshrc
alias cr='cargo run'
alias cc='cargo check'
alias ct='cargo test'
alias cw='cargo watch -x run'
alias db='make shell-db'
```

### 2. Variables de Entorno

```bash
# .env local (no commitear)
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/hodei_policies
RUST_LOG=info
RUST_BACKTRACE=1  # Para ver stack traces completos
```

### 3. Snippets VSCode

```json
{
  "Derive HodeiEntity": {
    "prefix": "hodei-entity",
    "body": [
      "#[derive(Debug, Serialize, Deserialize, Clone, HodeiEntity, sqlx::FromRow)]",
      "#[hodei(entity_type = \"HodeiMVP::${1:EntityName}\")]",
      "pub struct ${1:EntityName} {",
      "    pub id: Hrn,",
      "    $0",
      "}"
    ]
  }
}
```

### 4. Git Hooks

```bash
# .git/hooks/pre-commit
#!/bin/bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

---

## 🎯 Checklist de Desarrollo

Antes de hacer commit:

- [ ] `cargo fmt` - Código formateado
- [ ] `cargo clippy` - Sin warnings
- [ ] `cargo test` - Tests pasan
- [ ] `make test` - Tests de API pasan
- [ ] `cargo check` - Compila sin errores
- [ ] Documentación actualizada si es necesario

---

## 🆘 Problemas Comunes

### "Error connecting to database"

```bash
# Verificar que PostgreSQL esté corriendo
docker ps | grep postgres

# Si no está corriendo
make dev-up

# Verificar conexión
make shell-db
```

### "Port 5432 already in use"

```bash
# Ver qué está usando el puerto
lsof -i :5432

# Detener PostgreSQL local si existe
sudo systemctl stop postgresql

# O cambiar puerto en docker-compose.dev.yml
ports:
  - "5433:5432"  # Usar puerto 5433 en host
```

### "Compilation is slow"

```bash
# Usar cargo check en lugar de build
cargo check

# Usar cargo-watch
cargo watch -x check

# Limpiar y recompilar
cargo clean
cargo build
```

### "Schema not updating"

```bash
# Recompilar con schema-discovery
cargo clean
cargo build --features schema-discovery

# Verificar
cat cedar_schema.json | jq .
```

---

## 📚 Recursos

- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Rust Analyzer](https://rust-analyzer.github.io/)
- [cargo-watch](https://github.com/watchexec/cargo-watch)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)

---

**¡Feliz desarrollo! 🦀✨**
