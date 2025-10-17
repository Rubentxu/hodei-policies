# 🔍 Auditoría REST API - Problemas Encontrados

## ❌ Problemas Identificados

### 1. **CREATE Policy - Debería retornar 201 Created**
**Actual**: `200 OK`
**Debería ser**: `201 Created` + `Location` header

### 2. **CREATE Policy - Formato de respuesta inconsistente**
**Actual**:
```json
{
  "policy_id": "uuid",
  "message": "Policy created successfully"
}
```
**Problema**: Incluye un campo `message` innecesario. REST debe retornar el recurso creado.

**Debería ser**:
```json
{
  "id": "uuid",
  "content": "permit(...)"
}
```

### 3. **UPDATE Policy - Debería retornar el recurso actualizado**
**Actual**: Retorna `{"policy_id": "...", "message": "..."}`
**Debería ser**: Retornar el recurso completo actualizado

### 4. **LIST Policies - Formato de respuesta con wrapper innecesario**
**Actual**:
```json
{
  "policies": [...],
  "count": 10
}
```
**Problema**: El wrapper `policies` es redundante. El endpoint ya es `/policies`.

**Debería ser** (opción 1 - Simple):
```json
[
  {"id": "...", "content": "..."},
  {"id": "...", "content": "..."}
]
```

**O** (opción 2 - Con metadata de paginación):
```json
{
  "data": [...],
  "total": 10,
  "page": 1,
  "per_page": 20
}
```

### 5. **Nombres de campos inconsistentes**
- Policies usan `policy_id` 
- Documents/Artifacts usan `id`
- **Debería ser**: Siempre `id` para consistencia

### 6. **Mensajes en respuestas exitosas**
**Problema**: Los campos `message` no son necesarios en APIs REST.
- El código HTTP ya indica éxito
- El cuerpo debe contener datos, no mensajes

### 7. **Formato de error inconsistente**
**Actual**: Tupla `(StatusCode, String)`
**Debería ser**: Estructura consistente con `AppError`

---

## ✅ Correcciones Necesarias

### 1. CREATE Policy
```rust
// Antes
Ok(Json(serde_json::json!({
    "policy_id": policy_id,
    "message": "Policy created successfully"
})))

// Después
let location = format!("/_api/policies/{}", policy_id);
Ok((
    StatusCode::CREATED,
    [(axum::http::header::LOCATION, location)],
    Json(serde_json::json!({
        "id": policy_id,
        "content": policy_content
    }))
))
```

### 2. UPDATE Policy
```rust
// Antes
Ok(Json(serde_json::json!({
    "policy_id": id,
    "message": "Policy updated successfully"
})))

// Después
let policy = state.auth_service.get_policy(id.clone()).await?
    .ok_or(AppError::NotFound("Policy not found".into()))?;
    
Ok(Json(serde_json::json!({
    "id": id,
    "content": policy
})))
```

### 3. LIST Policies
```rust
// Opción Simple (recomendada para APIs pequeñas)
Ok(Json(policies_json))

// O con metadata (recomendada para APIs con paginación)
Ok(Json(serde_json::json!({
    "data": policies_json,
    "total": policies_json.len()
})))
```

### 4. DELETE Policy
```rust
// Ya está correcto - 204 No Content
Ok(StatusCode::NO_CONTENT)
```

---

## 📋 Estándares REST a Seguir

### Códigos de Estado
- ✅ `200 OK` - GET, PUT exitoso
- ✅ `201 Created` - POST que crea recurso
- ✅ `204 No Content` - DELETE exitoso
- ✅ `400 Bad Request` - Validación fallida
- ✅ `404 Not Found` - Recurso no existe
- ✅ `409 Conflict` - Conflicto (ej: recurso ya existe)

### Formato de Respuestas

#### Recurso Individual
```json
{
  "id": "123",
  "field1": "value1",
  "field2": "value2"
}
```

#### Lista de Recursos (Simple)
```json
[
  {"id": "1", "field": "value"},
  {"id": "2", "field": "value"}
]
```

#### Lista con Paginación
```json
{
  "data": [...],
  "total": 100,
  "page": 1,
  "per_page": 20
}
```

#### Error
```json
{
  "error": "Mensaje descriptivo"
}
```

### Nombres de Campos
- ✅ Usar `snake_case` en JSON
- ✅ Usar `id` (no `policy_id`, `document_id`, etc.)
- ✅ Ser consistente en toda la API

### Headers
- ✅ `Content-Type: application/json`
- ✅ `Location` en respuestas 201 Created
- ✅ `Cache-Control` para recursos cacheables

---

## 🎯 Plan de Corrección

1. ✅ Actualizar `create_policy_handler` - 201 + Location
2. ✅ Actualizar `update_policy_handler` - Retornar recurso
3. ✅ Actualizar `list_policies_handler` - Formato simple
4. ✅ Estandarizar nombres de campos a `id`
5. ✅ Remover campos `message` innecesarios
6. ✅ Verificar consistencia en toda la API
