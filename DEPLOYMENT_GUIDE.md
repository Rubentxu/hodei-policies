# Guía de Despliegue - Hodei Authorization Engine

## 🚀 Inicio Rápido (Quick Start)

### Opción 1: Script Automático

```bash
./quick-start.sh
```

Este script:
1. ✅ Verifica Docker y Docker Compose
2. ✅ Levanta PostgreSQL y la aplicación
3. ✅ Espera a que los servicios estén listos
4. ✅ Ejecuta todos los tests de API
5. ✅ Muestra el resultado

### Opción 2: Comandos Manuales

```bash
# 1. Levantar servicios
make docker-up

# 2. Esperar a que estén listos (10-15 segundos)
sleep 10

# 3. Ejecutar tests
make test

# 4. Ver logs si hay problemas
make logs-app
```

---

## 📋 Requisitos del Sistema

### Software Requerido

- **Docker**: 20.10+
- **Docker Compose**: 2.0+
- **Make**: (opcional, para comandos simplificados)
- **curl**: Para ejecutar tests
- **jq**: (opcional, para visualizar JSON)

### Verificar Instalación

```bash
docker --version
docker-compose --version
make --version
curl --version
```

---

## 🏗️ Arquitectura de Despliegue

```
┌─────────────────────────────────────────┐
│         Docker Compose                   │
├─────────────────────────────────────────┤
│                                          │
│  ┌──────────────┐    ┌──────────────┐  │
│  │              │    │              │  │
│  │  PostgreSQL  │◄───┤  Hodei App   │  │
│  │   :5432      │    │   :3000      │  │
│  │              │    │              │  │
│  └──────────────┘    └──────────────┘  │
│         │                    │          │
│         │                    │          │
│    [postgres_data]      [cedar_schema] │
│                                          │
└─────────────────────────────────────────┘
```

---

## 🔧 Configuración

### Variables de Entorno

Archivo `.env`:

```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/hodei_policies
RUST_LOG=info
```

Para Docker, las variables se configuran en `docker-compose.yml`.

### Puertos

| Servicio   | Puerto | Descripción                |
|------------|--------|----------------------------|
| PostgreSQL | 5432   | Base de datos              |
| Hodei App  | 3000   | API REST                   |

---

## 📦 Proceso de Despliegue

### 1. Preparación

```bash
# Clonar repositorio
git clone <repo-url>
cd hodei-policies

# Verificar archivos necesarios
ls -la docker-compose.yml Dockerfile
```

### 2. Construcción

```bash
# Construir imagen Docker
docker-compose build

# O forzar reconstrucción
docker-compose build --no-cache
```

### 3. Despliegue

```bash
# Levantar servicios en background
docker-compose up -d

# Ver logs en tiempo real
docker-compose logs -f
```

### 4. Verificación

```bash
# Verificar servicios corriendo
docker-compose ps

# Verificar salud de PostgreSQL
docker-compose exec postgres pg_isready -U postgres

# Verificar salud de la aplicación
curl http://localhost:3000/health
```

### 5. Tests

```bash
# Ejecutar suite completa de tests
./tests/api_tests.sh

# O con make
make test
```

---

## 🧪 Tests de Validación

### Suite de Tests Incluida

El script `tests/api_tests.sh` valida:

1. **REQ-HRN-04**: Hidratación de HRN
   - ✅ Crear documento con ID simple
   - ✅ Backend construye HRN completo

2. **REQ-PM-01**: Aislamiento Multi-Tenant
   - ✅ Alice (tenant-a) accede a sus documentos
   - ✅ Bob (tenant-b) NO accede a documentos de Alice
   - ✅ Aislamiento estricto entre tenants

3. **REQ-SVC-05**: Flujo de Autorización
   - ✅ Extraer contexto del token
   - ✅ Hidratar HRN con tenant_id
   - ✅ Cargar entidades de BD
   - ✅ Autorizar con Cedar
   - ✅ Ejecutar operación

4. **REQ-API-01**: Gestión de Políticas
   - ✅ Agregar política dinámicamente
   - ✅ Política se aplica inmediatamente

5. **REQ-DM-01**: Recursos Virtuales
   - ✅ Crear documento (recurso virtual)
   - ✅ Autorización antes de persistir

### Ejecutar Tests

```bash
# Tests completos
./tests/api_tests.sh

# Tests con output detallado
bash -x ./tests/api_tests.sh

# Tests contra URL específica
API_URL=http://production:3000 ./tests/api_tests.sh
```

### Resultado Esperado

```
🧪 Test: Alice (admin, tenant-a) crea documento doc-test1 ... ✓ PASS (HTTP 200)
🧪 Test: Alice (tenant-a) lee su propio documento ... ✓ PASS (HTTP 200)
🧪 Test: Bob (tenant-b) NO puede leer documento de Alice (tenant-a) ... ✓ PASS (HTTP 403)
...
✓ Tests Pasados: 15
✗ Tests Fallidos: 0
🎉 ¡Todos los tests pasaron exitosamente!
```

---

## 🐛 Troubleshooting

### Problema: Servicios no inician

```bash
# Ver logs detallados
docker-compose logs

# Verificar puertos disponibles
lsof -i :3000
lsof -i :5432

# Limpiar y reiniciar
docker-compose down -v
docker-compose up -d
```

### Problema: PostgreSQL no responde

```bash
# Verificar estado
docker-compose ps postgres

# Ver logs de PostgreSQL
docker-compose logs postgres

# Reiniciar solo PostgreSQL
docker-compose restart postgres

# Verificar conexión
docker-compose exec postgres psql -U postgres -c "SELECT 1"
```

### Problema: Aplicación no conecta a BD

```bash
# Verificar variables de entorno
docker-compose exec app env | grep DATABASE

# Verificar red Docker
docker network inspect hodei-policies_default

# Reiniciar aplicación
docker-compose restart app
```

### Problema: Tests fallan

```bash
# Verificar que servicios estén listos
curl http://localhost:3000/health

# Esperar más tiempo
sleep 15 && ./tests/api_tests.sh

# Ver logs de la aplicación
docker-compose logs app | tail -50

# Ejecutar test individual
curl -X POST http://localhost:3000/documents \
  -H "Authorization: Bearer alice" \
  -H "Content-Type: application/json" \
  -d '{"resource_id":"test","is_public":false}'
```

### Problema: Esquema Cedar no se carga

```bash
# Verificar que el archivo existe
docker-compose exec app ls -la /app/cedar_schema.json

# Ver contenido del esquema
docker-compose exec app cat /app/cedar_schema.json

# Regenerar esquema localmente
make schema

# Reconstruir imagen
docker-compose build --no-cache app
```

---

## 🔄 Actualización y Mantenimiento

### Actualizar Aplicación

```bash
# 1. Detener servicios
docker-compose down

# 2. Actualizar código
git pull

# 3. Reconstruir imagen
docker-compose build

# 4. Levantar servicios
docker-compose up -d

# 5. Verificar
make test
```

### Backup de Base de Datos

```bash
# Crear backup
docker-compose exec postgres pg_dump -U postgres hodei_policies > backup.sql

# Restaurar backup
docker-compose exec -T postgres psql -U postgres hodei_policies < backup.sql
```

### Ver Logs

```bash
# Logs de todos los servicios
docker-compose logs -f

# Solo aplicación
docker-compose logs -f app

# Solo PostgreSQL
docker-compose logs -f postgres

# Últimas 100 líneas
docker-compose logs --tail=100 app
```

### Limpiar Sistema

```bash
# Detener y eliminar contenedores
docker-compose down

# Eliminar también volúmenes (¡CUIDADO! Borra datos)
docker-compose down -v

# Limpiar imágenes no usadas
docker image prune -a
```

---

## 🔐 Seguridad en Producción

### Recomendaciones

1. **Cambiar Credenciales**
   ```yaml
   # docker-compose.yml
   environment:
     POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}  # Usar variable de entorno
   ```

2. **Usar TLS/SSL**
   ```bash
   DATABASE_URL=postgresql://user:pass@host:5432/db?sslmode=require
   ```

3. **Limitar Puertos**
   ```yaml
   # Solo exponer app, no PostgreSQL
   ports:
     - "3000:3000"
   # Comentar puerto de PostgreSQL
   ```

4. **Usar Secrets**
   ```yaml
   secrets:
     db_password:
       file: ./secrets/db_password.txt
   ```

5. **Configurar Firewall**
   ```bash
   # Solo permitir puerto 3000
   ufw allow 3000/tcp
   ```

---

## 📊 Monitoreo

### Health Check

```bash
# Verificar salud
curl http://localhost:3000/health

# Con timeout
curl --max-time 5 http://localhost:3000/health
```

### Métricas

```bash
# Ver uso de recursos
docker stats

# Ver logs de errores
docker-compose logs app | grep ERROR

# Ver conexiones activas
docker-compose exec postgres psql -U postgres -c \
  "SELECT count(*) FROM pg_stat_activity WHERE datname='hodei_policies'"
```

---

## 🚀 Despliegue en Producción

### Opción 1: Docker Compose (Simple)

```bash
# En servidor de producción
git clone <repo>
cd hodei-policies
cp .env.example .env
# Editar .env con credenciales seguras
docker-compose -f docker-compose.prod.yml up -d
```

### Opción 2: Kubernetes (Avanzado)

Ver `k8s/` para manifiestos de Kubernetes (próximamente).

### Opción 3: Cloud (AWS/GCP/Azure)

- Usar RDS/Cloud SQL para PostgreSQL
- Desplegar app en ECS/Cloud Run/App Service
- Configurar load balancer y auto-scaling

---

## 📞 Soporte

Si encuentras problemas:

1. Revisa los logs: `make logs-app`
2. Verifica la documentación: `README.md`
3. Consulta troubleshooting arriba
4. Abre un issue en GitHub

---

**¡Listo para producción! 🎉**
