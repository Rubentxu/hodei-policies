# Guía de Administración de Base de Datos

## 🗄️ Herramientas de Administración Incluidas

El proyecto incluye **dos herramientas** para gestionar PostgreSQL:

### 1. Adminer (Recomendado - Ligero y Rápido)

**Características:**
- ✅ Interfaz simple y rápida
- ✅ Solo 1 archivo PHP (~500KB)
- ✅ Carga instantánea
- ✅ Soporta múltiples bases de datos
- ✅ Tema oscuro incluido

**Acceso:**
- URL: http://localhost:8080
- Sistema: PostgreSQL
- Servidor: `postgres`
- Usuario: `postgres`
- Contraseña: `postgres`
- Base de datos: `hodei_policies`

### 2. pgAdmin (Completo - Más Funciones)

**Características:**
- ✅ Interfaz completa y profesional
- ✅ Editor SQL avanzado
- ✅ Visualización de esquemas
- ✅ Herramientas de backup/restore
- ✅ Monitoreo de rendimiento

**Acceso:**
- URL: http://localhost:5050
- Email: `admin@hodei.com`
- Contraseña: `admin`

**Primera vez - Configurar conexión:**
1. Click derecho en "Servers" → "Register" → "Server"
2. Pestaña "General":
   - Name: `Hodei PostgreSQL`
3. Pestaña "Connection":
   - Host: `postgres`
   - Port: `5432`
   - Database: `hodei_policies`
   - Username: `postgres`
   - Password: `postgres`
4. Click "Save"

---

## 🚀 Inicio Rápido

### Levantar Todos los Servicios

```bash
# Levantar PostgreSQL + App + Adminer + pgAdmin
docker-compose up -d

# Verificar que estén corriendo
docker-compose ps
```

### Acceder a las Herramientas

```bash
# Abrir Adminer (más rápido)
open http://localhost:8080

# Abrir pgAdmin (más completo)
open http://localhost:5050
```

---

## 📊 Consultas Útiles

### Ver Todas las Tablas

```sql
SELECT table_name 
FROM information_schema.tables 
WHERE table_schema = 'public';
```

### Ver Usuarios

```sql
SELECT * FROM users;
```

Resultado esperado:
```
id                                                          | role
------------------------------------------------------------+-------
hrn:aws:users-api:eu-west-1:tenant-a:user/alice           | admin
hrn:aws:users-api:eu-west-1:tenant-b:user/bob             | user
```

### Ver Documentos

```sql
SELECT * FROM documents;
```

### Ver Políticas

```sql
SELECT id, content FROM policies;
```

### Ver Estructura de una Tabla

```sql
SELECT 
    column_name, 
    data_type, 
    is_nullable
FROM information_schema.columns
WHERE table_name = 'documents';
```

---

## 🔍 Explorar Datos

### Filtrar por Tenant

```sql
-- Documentos de tenant-a
SELECT * FROM documents 
WHERE id LIKE '%tenant-a%';

-- Usuarios de tenant-b
SELECT * FROM users 
WHERE id LIKE '%tenant-b%';
```

### Buscar por Owner

```sql
SELECT 
    d.id as document_id,
    d.owner_id,
    d.is_public,
    u.role as owner_role
FROM documents d
JOIN users u ON d.owner_id = u.id;
```

### Contar Recursos por Tenant

```sql
-- Contar documentos por tenant
SELECT 
    CASE 
        WHEN id LIKE '%tenant-a%' THEN 'tenant-a'
        WHEN id LIKE '%tenant-b%' THEN 'tenant-b'
        ELSE 'other'
    END as tenant,
    COUNT(*) as total
FROM documents
GROUP BY tenant;
```

---

## 🛠️ Operaciones Comunes

### Limpiar Datos de Test

```sql
-- Eliminar documentos de test
DELETE FROM documents 
WHERE id LIKE '%doc-test%';

-- Verificar
SELECT COUNT(*) FROM documents;
```

### Resetear Base de Datos

```sql
-- CUIDADO: Esto elimina TODOS los datos
TRUNCATE TABLE documents CASCADE;
TRUNCATE TABLE users CASCADE;
TRUNCATE TABLE policies CASCADE;

-- Reiniciar la aplicación para que haga seed
-- docker-compose restart app
```

### Agregar Usuario de Prueba

```sql
INSERT INTO users (id, role) 
VALUES (
    'hrn:aws:users-api:eu-west-1:tenant-c:user/charlie',
    'user'
);
```

### Agregar Documento de Prueba

```sql
INSERT INTO documents (id, owner_id, is_public) 
VALUES (
    'hrn:aws:documents-api:eu-west-1:tenant-a:document/test123',
    'hrn:aws:users-api:eu-west-1:tenant-a:user/alice',
    false
);
```

---

## 📈 Monitoreo

### Ver Conexiones Activas

```sql
SELECT 
    pid,
    usename,
    application_name,
    client_addr,
    state,
    query
FROM pg_stat_activity
WHERE datname = 'hodei_policies';
```

### Ver Tamaño de Tablas

```sql
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

### Ver Índices

```sql
SELECT 
    tablename,
    indexname,
    indexdef
FROM pg_indexes
WHERE schemaname = 'public';
```

---

## 🔧 Administración

### Backup de Base de Datos

```bash
# Desde la línea de comandos
docker-compose exec postgres pg_dump -U postgres hodei_policies > backup.sql

# O con make
make backup-db
```

### Restaurar Backup

```bash
# Desde la línea de comandos
docker-compose exec -T postgres psql -U postgres hodei_policies < backup.sql

# O con make
make restore-db FILE=backup.sql
```

### Ejecutar SQL desde Archivo

```bash
# Crear archivo queries.sql con tus consultas
docker-compose exec -T postgres psql -U postgres hodei_policies < queries.sql
```

### Acceder a psql Directamente

```bash
# Con make
make shell-db

# O directamente
docker-compose exec postgres psql -U postgres -d hodei_policies
```

---

## 🎨 Personalizar Adminer

### Cambiar Tema

Adminer incluye varios temas. Para cambiar:

```yaml
# En docker-compose.yml
adminer:
  environment:
    ADMINER_DESIGN: pepa-linha-dark  # Tema oscuro (actual)
    # Otras opciones:
    # ADMINER_DESIGN: pepa-linha      # Tema claro
    # ADMINER_DESIGN: nette           # Tema Nette
    # ADMINER_DESIGN: hydra           # Tema Hydra
```

### Plugins de Adminer

Para agregar plugins, crear un `Dockerfile` personalizado:

```dockerfile
FROM adminer:latest
RUN mkdir -p /var/www/html/plugins-enabled
COPY adminer-plugins/* /var/www/html/plugins-enabled/
```

---

## 🔐 Seguridad

### Cambiar Credenciales en Producción

**IMPORTANTE**: Las credenciales por defecto son solo para desarrollo.

```yaml
# docker-compose.yml
postgres:
  environment:
    POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}  # Usar variable de entorno

pgadmin:
  environment:
    PGADMIN_DEFAULT_EMAIL: ${PGADMIN_EMAIL}
    PGADMIN_DEFAULT_PASSWORD: ${PGADMIN_PASSWORD}
```

### Deshabilitar Herramientas en Producción

```bash
# Levantar solo PostgreSQL y App (sin Adminer ni pgAdmin)
docker-compose up -d postgres app
```

O comentar los servicios en `docker-compose.yml`.

---

## 📱 Comandos Rápidos

```bash
# Ver logs de PostgreSQL
docker-compose logs -f postgres

# Ver tamaño de la base de datos
docker-compose exec postgres psql -U postgres -c \
  "SELECT pg_size_pretty(pg_database_size('hodei_policies'));"

# Contar registros en todas las tablas
docker-compose exec postgres psql -U postgres -d hodei_policies -c \
  "SELECT 'users' as table, COUNT(*) FROM users
   UNION ALL
   SELECT 'documents', COUNT(*) FROM documents
   UNION ALL
   SELECT 'policies', COUNT(*) FROM policies;"

# Verificar versión de PostgreSQL
docker-compose exec postgres psql -U postgres -c "SELECT version();"
```

---

## 🆘 Troubleshooting

### Adminer no carga

```bash
# Verificar que el contenedor esté corriendo
docker-compose ps adminer

# Ver logs
docker-compose logs adminer

# Reiniciar
docker-compose restart adminer
```

### pgAdmin no conecta a PostgreSQL

1. Verificar que el host sea `postgres` (no `localhost`)
2. Verificar que el puerto sea `5432`
3. Verificar credenciales
4. Ver logs: `docker-compose logs pgadmin`

### Error de permisos en pgAdmin

```bash
# Eliminar volumen y recrear
docker-compose down
docker volume rm hodei-policies_pgadmin_data
docker-compose up -d pgadmin
```

---

## 📚 Recursos Adicionales

- [Adminer Documentation](https://www.adminer.org/)
- [pgAdmin Documentation](https://www.pgadmin.org/docs/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)

---

**¡Disfruta gestionando tu base de datos!** 🗄️
