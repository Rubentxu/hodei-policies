# Hodei - Motor de Autorización con Cedar Policy

Motor de autorización avanzado construido en Rust con Cedar Policy, diseñado para aplicaciones multi-tenant seguras y escalables.

## 🎯 Características Principales

- **Schema-as-Code**: El esquema Cedar se genera automáticamente desde el código Rust
- **Multi-Tenancy Nativo**: Aislamiento estricto entre tenants mediante HRN
- **Identidad Fuerte**: Hodei Resource Names (HRN) como identificadores únicos
- **Arquitectura Desacoplada**: Servicios modulares con inversión de dependencias
- **Políticas Dinámicas**: Gestión de políticas en runtime sin reinicio

## 🏗️ Arquitectura

```
hodei-authz-sdk-policies/
├── kernel/                    # Tipo Hrn (identidad fuerte)
├── hodei_provider/            # Traits y colección de esquema
├── hodei_provider_derive/     # Macros procedurales
├── hodei_domain/              # Modelos de dominio
├── src/
│   ├── auth.rs               # Servicio de autorización
│   ├── mapper.rs             # Servicio de mapeo genérico
│   └── main.rs               # Handlers Axum
└── migrations/                # Migraciones PostgreSQL
```

## 🚀 Inicio Rápido

### Prerequisitos

- Docker y Docker Compose
- Rust 1.75+ (opcional, para desarrollo local)
- Make (opcional, para comandos simplificados)

### Opción 1: Con Docker (Recomendado)

```bash
# Levantar todos los servicios
make docker-up

# Ejecutar tests de API
make test

# Ver logs
make docker-logs

# Detener servicios
make docker-down
```

### Opción 2: Desarrollo Local

```bash
# 1. Configurar entorno
cp .env.example .env

# 2. Levantar solo PostgreSQL
docker-compose up -d postgres

# 3. Ejecutar migraciones
make migrate

# 4. Compilar y ejecutar
make run

# 5. En otra terminal, ejecutar tests
make test
```

## 🧪 Tests de API

Los tests validan todos los requisitos arquitectónicos:

```bash
# Ejecutar tests
./tests/api_tests.sh

# O con make
make test
```

### Requisitos Validados

- ✅ **REQ-HRN-04**: Hidratación de HRN en backend
- ✅ **REQ-PM-01**: Aislamiento multi-tenant estricto
- ✅ **REQ-SVC-05**: Flujo de autorización completo
- ✅ **REQ-API-01**: Gestión dinámica de políticas
- ✅ **REQ-DM-01**: Recursos virtuales en creación

## 📋 Comandos Disponibles

```bash
make help              # Ver todos los comandos
make build             # Compilar aplicación
make run               # Ejecutar localmente
make test              # Ejecutar tests de API
make docker-up         # Levantar servicios Docker
make docker-down       # Detener servicios
make docker-test       # Test completo con Docker
make migrate           # Ejecutar migraciones
make schema            # Regenerar esquema Cedar
make logs-app          # Ver logs de la aplicación
make logs-db           # Ver logs de PostgreSQL
make shell-db          # Abrir psql
```

## 🔧 Configuración

### Variables de Entorno

```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/hodei_policies
RUST_LOG=info
```

### Usuarios de Prueba

El sistema incluye usuarios pre-configurados:

| Token   | Role  | Tenant   | Descripción           |
|---------|-------|----------|-----------------------|
| `alice` | admin | tenant-a | Administrador tenant A |
| `bob`   | user  | tenant-b | Usuario tenant B       |

## 📡 API Endpoints

### Documentos

```bash
# Crear documento
POST /documents
Authorization: Bearer {token}
Content-Type: application/json
{
  "resource_id": "doc1",
  "is_public": false
}

# Leer documento
GET /documents/{resource_id}
Authorization: Bearer {token}

# Actualizar documento
PUT /documents/{resource_id}
Authorization: Bearer {token}
Content-Type: application/json
{
  "is_public": true
}

# Eliminar documento
DELETE /documents/{resource_id}
Authorization: Bearer {token}
```

### Políticas

```bash
# Agregar/actualizar política
POST /_api/policies/{policy_id}
Content-Type: text/plain

permit(principal, action, resource) when { ... };
```

## 🧩 Ejemplos de Uso

### Crear un Documento

```bash
curl -X POST http://localhost:3000/documents \
  -H "Authorization: Bearer alice" \
  -H "Content-Type: application/json" \
  -d '{"resource_id":"my-doc","is_public":false}'
```

### Leer un Documento

```bash
curl http://localhost:3000/documents/my-doc \
  -H "Authorization: Bearer alice"
```

### Agregar una Política

```bash
curl -X POST http://localhost:3000/_api/policies/my_policy \
  -H "Content-Type: text/plain" \
  -d 'permit(principal, action, resource) when { principal.role == "admin" };'
```

## 🔐 Políticas de Ejemplo

### Aislamiento de Tenant

```cedar
forbid(principal, action, resource) 
unless { 
  principal.tenant_id == resource.tenant_id 
};
```

### Permisos de Propietario

```cedar
permit(principal, action, resource) 
when { 
  resource.owner_id == principal.id 
};
```

### Permisos de Admin

```cedar
permit(principal, action == Action::"Create", resource) 
when { 
  principal.role == "admin" 
};
```

## 📚 Agregar Nuevas Entidades

El sistema usa **schema-as-code**. Para agregar una nueva entidad:

```rust
// En hodei_domain/src/lib.rs

#[derive(Debug, Serialize, Deserialize, Clone, HodeiEntity, sqlx::FromRow)]
#[hodei-authz-sdk(entity_type = "HodeiMVP::Project")]
pub struct Project {
    pub id: Hrn,
    pub name: String,
    pub owner_id: Hrn,
}

#[derive(Debug, Clone, HodeiAction)]
#[hodei-authz-sdk(namespace = "HodeiMVP")]
pub enum ProjectCommand {
    #[hodei-authz-sdk(principal = "User", resource = "Project", creates_resource)]
    Create(ProjectCreatePayload),
    #[hodei-authz-sdk(principal = "User", resource = "Project")]
    Read { id: Hrn },
}
```

El esquema Cedar se actualizará automáticamente en la próxima compilación.

## 🐛 Troubleshooting

### La aplicación no se conecta a PostgreSQL

```bash
# Verificar que PostgreSQL esté corriendo
docker-compose ps

# Ver logs de PostgreSQL
make logs-db

# Reiniciar servicios
make docker-down && make docker-up
```

### Tests fallan

```bash
# Verificar que la aplicación esté corriendo
curl http://localhost:3000/health || echo "App no responde"

# Ver logs de la aplicación
make logs-app

# Esperar más tiempo para que los servicios inicien
sleep 10 && make test
```

### Esquema Cedar no se genera

```bash
# Regenerar esquema
make schema

# Verificar que hodei_domain esté enlazado en build.rs
grep "use hodei_domain" build.rs
```

## 📖 Documentación Adicional

- [IMPLEMENTATION_SUMMARY.md](./IMPLEMENTATION_SUMMARY.md) - Resumen de implementación
- [FIXES_APPLIED.md](./FIXES_APPLIED.md) - Correcciones y soluciones
- [Documento de Requisitos](./REQUIREMENTS.md) - Requisitos completos

## 🤝 Contribuir

1. Fork el proyecto
2. Crea una rama para tu feature (`git checkout -b feature/amazing-feature`)
3. Commit tus cambios (`git commit -m 'Add amazing feature'`)
4. Push a la rama (`git push origin feature/amazing-feature`)
5. Abre un Pull Request

## 📝 Licencia

Este proyecto está bajo la licencia MIT.

## 🙏 Agradecimientos

- [Cedar Policy](https://www.cedarpolicy.com/) - Motor de políticas
- [SQLx](https://github.com/launchbadge/sqlx) - SQL toolkit async
- [Axum](https://github.com/tokio-rs/axum) - Framework web
- [Inventory](https://github.com/dtolnay/inventory) - Plugin registry

---

**Desarrollado con ❤️ usando Rust**
