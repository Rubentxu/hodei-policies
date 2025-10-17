# API CRUD de Políticas - Gestión de Ciclo de Vida

## 🎯 Inspirado en AWS Verified Permissions

Implementación completa de gestión de políticas con UUIDs únicos, similar a AWS Verified Permissions.

---

## 📋 Endpoints Disponibles

### 1. Crear Política

**POST** `/_api/policies`

Crea una nueva política con un UUID único generado automáticamente.

**Request**:
```bash
curl -X POST http://localhost:3000/_api/policies \
  -H "Content-Type: text/plain" \
  -d 'permit(principal, action, resource) when { principal.role == "admin" };'
```

**Response** (201 Created):
```json
{
  "policy_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "message": "Policy created successfully"
}
```

**Características**:
- ✅ UUID único generado automáticamente
- ✅ Agrega `@id("uuid")` automáticamente si no existe
- ✅ Recarga políticas inmediatamente (hot-reload)

---

### 2. Listar Políticas

**GET** `/_api/policies`

Obtiene todas las políticas existentes.

**Request**:
```bash
curl http://localhost:3000/_api/policies
```

**Response** (200 OK):
```json
{
  "policies": [
    {
      "id": "tenant_isolation",
      "content": "@id(\"tenant_isolation\")\nforbid(principal, action, resource) unless { principal.tenant_id == resource.tenant_id };"
    },
    {
      "id": "owner_permissions",
      "content": "@id(\"owner_permissions\")\npermit(principal, action, resource) when { resource.owner_id == principal.id };"
    }
  ],
  "count": 2
}
```

---

### 3. Obtener Política por ID

**GET** `/_api/policies/:id`

Obtiene una política específica por su ID.

**Request**:
```bash
curl http://localhost:3000/_api/policies/tenant_isolation
```

**Response** (200 OK):
```json
{
  "id": "tenant_isolation",
  "content": "@id(\"tenant_isolation\")\nforbid(principal, action, resource) unless { principal.tenant_id == resource.tenant_id };"
}
```

**Response** (404 Not Found):
```json
"Policy not found"
```

---

### 4. Actualizar Política

**PUT** `/_api/policies/:id`

Actualiza una política existente.

**Request**:
```bash
curl -X PUT http://localhost:3000/_api/policies/tenant_isolation \
  -H "Content-Type: text/plain" \
  -d 'forbid(principal, action, resource) unless { 
    principal.tenant_id == resource.tenant_id && 
    resource.archived == false 
  };'
```

**Response** (200 OK):
```json
{
  "policy_id": "tenant_isolation",
  "message": "Policy updated successfully"
}
```

**Response** (404 Not Found):
```json
"Policy ID not found for removal"
```

**Características**:
- ✅ Actualiza el contenido de la política
- ✅ Mantiene el mismo ID
- ✅ Recarga políticas inmediatamente

---

### 5. Eliminar Política

**DELETE** `/_api/policies/:id`

Elimina una política por su ID.

**Request**:
```bash
curl -X DELETE http://localhost:3000/_api/policies/a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

**Response** (204 No Content):
```
(Sin contenido)
```

**Response** (404 Not Found):
```json
"Policy ID not found for removal"
```

**Características**:
- ✅ Elimina permanentemente la política
- ✅ Recarga políticas inmediatamente
- ✅ Retorna 404 si no existe

---

## 🔄 Flujo de Gestión de Ciclo de Vida

```
┌─────────────────────────────────────────────────────────┐
│                  CRUD de Políticas                       │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
    ┌──────────────────────────────────────────────┐
    │  1. CREATE - Generar UUID único              │
    │     POST /_api/policies                      │
    │     → policy_id: "uuid-1234..."              │
    └──────────────────────────────────────────────┘
                          │
                          ▼
    ┌──────────────────────────────────────────────┐
    │  2. READ - Obtener política(s)               │
    │     GET /_api/policies                       │
    │     GET /_api/policies/:id                   │
    └──────────────────────────────────────────────┘
                          │
                          ▼
    ┌──────────────────────────────────────────────┐
    │  3. UPDATE - Modificar política              │
    │     PUT /_api/policies/:id                   │
    │     → Mantiene mismo ID                      │
    └──────────────────────────────────────────────┘
                          │
                          ▼
    ┌──────────────────────────────────────────────┐
    │  4. DELETE - Eliminar política               │
    │     DELETE /_api/policies/:id                │
    │     → Eliminación permanente                 │
    └──────────────────────────────────────────────┘
                          │
                          ▼
    ┌──────────────────────────────────────────────┐
    │  5. HOT-RELOAD - Recarga automática          │
    │     PolicySet se actualiza inmediatamente    │
    │     Sin necesidad de reiniciar               │
    └──────────────────────────────────────────────┘
```

---

## 🎨 Ejemplos de Uso

### Ejemplo 1: Crear Política de Lectura Pública

```bash
# 1. Crear política
POLICY_ID=$(curl -s -X POST http://localhost:3000/_api/policies \
  -H "Content-Type: text/plain" \
  -d 'permit(principal, action == Action::"Read", resource) when { resource.is_public == true };' \
  | jq -r '.policy_id')

echo "Política creada con ID: $POLICY_ID"

# 2. Verificar que se creó
curl http://localhost:3000/_api/policies/$POLICY_ID | jq .

# 3. Probar que funciona (intentar leer documento público)
curl http://localhost:3000/documents/doc-public \
  -H "Authorization: Bearer bob"
```

### Ejemplo 2: Actualizar Política Existente

```bash
# 1. Listar políticas actuales
curl http://localhost:3000/_api/policies | jq '.policies[] | .id'

# 2. Actualizar política de tenant isolation
curl -X PUT http://localhost:3000/_api/policies/tenant_isolation \
  -H "Content-Type: text/plain" \
  -d 'forbid(principal, action, resource) unless { 
    principal.tenant_id == resource.tenant_id && 
    principal.active == true 
  };'

# 3. Verificar actualización
curl http://localhost:3000/_api/policies/tenant_isolation | jq .
```

### Ejemplo 3: Eliminar Política Temporal

```bash
# 1. Crear política temporal
TEMP_ID=$(curl -s -X POST http://localhost:3000/_api/policies \
  -H "Content-Type: text/plain" \
  -d 'permit(principal, action, resource) when { context.debug_mode == true };' \
  | jq -r '.policy_id')

# 2. Usar la política...

# 3. Eliminar cuando ya no se necesite
curl -X DELETE http://localhost:3000/_api/policies/$TEMP_ID

# 4. Verificar eliminación
curl http://localhost:3000/_api/policies/$TEMP_ID
# → 404 Not Found
```

---

## 🔒 Mejores Prácticas

### 1. IDs de Políticas

**✅ Buenas prácticas**:
```bash
# UUIDs para políticas dinámicas (generadas por API)
a1b2c3d4-e5f6-7890-abcd-ef1234567890

# Nombres descriptivos para políticas base (seed)
tenant_isolation
owner_permissions
admin_creation
```

**❌ Evitar**:
```bash
# IDs genéricos sin significado
policy0, policy1, policy2

# Nombres de usuario en IDs
alice_policy, bob_policy
```

### 2. Versionado de Políticas

```bash
# Crear nueva versión
curl -X POST http://localhost:3000/_api/policies \
  -d '@id("tenant_isolation_v2")
forbid(principal, action, resource) unless { 
  principal.tenant_id == resource.tenant_id && 
  resource.archived == false 
};'

# Probar en staging

# Eliminar versión antigua
curl -X DELETE http://localhost:3000/_api/policies/tenant_isolation_v1
```

### 3. Políticas con Comentarios

```cedar
@id("admin_full_access")
// Política que otorga acceso completo a administradores
// Creada: 2025-10-17
// Autor: Security Team
permit(
    principal,
    action,
    resource
) when { 
    principal.role == "admin" 
};
```

### 4. Testing de Políticas

```bash
# 1. Crear política de test
TEST_ID=$(curl -s -X POST http://localhost:3000/_api/policies \
  -d 'permit(principal, action, resource) when { context.test_mode == true };' \
  | jq -r '.policy_id')

# 2. Ejecutar tests

# 3. Limpiar
curl -X DELETE http://localhost:3000/_api/policies/$TEST_ID
```

---

## 🔍 Comparación con AWS Verified Permissions

| Característica | AWS Verified Permissions | Hodei Implementation |
|----------------|-------------------------|----------------------|
| **IDs Únicos** | ✅ UUIDs | ✅ UUIDs (uuid v4) |
| **CRUD Completo** | ✅ Sí | ✅ Sí |
| **Hot-Reload** | ✅ Automático | ✅ Automático |
| **Versionado** | ✅ Policy Stores | ✅ Manual (IDs versionados) |
| **Validación** | ✅ Schema validation | ✅ Cedar validation |
| **Auditoría** | ✅ CloudTrail | 🔄 Por implementar |
| **Templates** | ✅ Policy Templates | 🔄 Por implementar |

---

## 🚀 Próximas Mejoras

1. **Auditoría**:
   ```rust
   // Registrar cambios en políticas
   audit_log.record(PolicyChange {
       policy_id,
       action: "CREATE",
       user: principal,
       timestamp: Utc::now()
   });
   ```

2. **Policy Templates**:
   ```rust
   // Plantillas reutilizables
   let template = PolicyTemplate::new("owner_access")
       .with_param("resource_type")
       .build();
   ```

3. **Validación Avanzada**:
   ```rust
   // Validar contra esquema antes de guardar
   validator.validate_policy(&policy, &schema)?;
   ```

4. **Dry-Run**:
   ```rust
   // Probar política sin aplicarla
   POST /_api/policies/dry-run
   ```

---

## 📚 Referencias

- [AWS Verified Permissions API](https://docs.aws.amazon.com/verifiedpermissions/latest/apireference/)
- [Cedar Policy Language](https://www.cedarpolicy.com/)
- [UUID RFC 4122](https://www.rfc-editor.org/rfc/rfc4122)

---

**Implementado**: 2025-10-17  
**Versión**: 1.0.0  
**Estado**: ✅ Producción Ready
