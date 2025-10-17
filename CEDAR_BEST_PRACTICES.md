# Cedar Policy - Mejores Prácticas

## 📚 Basado en AWS Verified Permissions y Cedar Documentation

### 1. Formato del Esquema (Cedar 4.x)

El esquema debe usar el formato correcto con `shape`:

```json
{
  "MyNamespace": {
    "entityTypes": {
      "User": {
        "memberOfTypes": [],
        "shape": {
          "type": "Record",
          "attributes": {
            "role": {
              "type": "String",
              "required": true
            },
            "email": {
              "type": "String",
              "required": false
            }
          }
        }
      }
    },
    "actions": {
      "Read": {
        "appliesTo": {
          "principalTypes": ["User"],
          "resourceTypes": ["Document"]
        }
      }
    }
  }
}
```

**✅ Correcto**: `shape` → `type: "Record"` → `attributes`
**❌ Incorrecto**: `attributes` directamente sin `shape`

### 2. Identificadores Únicos (UUIDs)

**Recomendación de AWS**: Usar UUIDs para todos los identificadores de principal y recursos.

**Razón**: Si un usuario "jane" deja la compañía y luego contratas a otra "jane", la nueva persona heredaría todos los permisos de la anterior.

**Implementación en Hodei**:
```rust
// ✅ CORRECTO - Usando HRN como identificador único
hrn:aws:users-api:eu-west-1:tenant-a:user/uuid-12345

// ❌ INCORRECTO - Usando nombres
hrn:aws:users-api:eu-west-1:tenant-a:user/jane
```

**Mejora sugerida**: Agregar comentarios amigables en políticas:
```cedar
permit(
    principal == User::"hrn:aws:users-api:eu-west-1:tenant-a:user/a1b2c3d4", // alice
    action,
    resource
);
```

### 3. Gestión de IDs de Políticas

**Problema actual**: Cedar asigna IDs automáticos (policy0, policy1) si no se especifican, causando conflictos al recargar.

**Solución AWS**: Cada política tiene un ID único (UUID) gestionado por el servicio.

**Solución Hodei**:

#### Opción A: IDs Explícitos en Políticas (Recomendado)
```cedar
@id("tenant_isolation_v1")
forbid(principal, action, resource) 
unless { principal.tenant_id == resource.tenant_id };
```

#### Opción B: Generar IDs al Cargar
```rust
// En load_policies
let policy_text = format!("@id(\"{}\")\n{}", db_id, content);
```

#### Opción C: PolicySet Dinámico
```rust
// Limpiar y recargar políticas en cada inicio
let mut policy_set = PolicySet::new();
// Cargar desde BD con IDs únicos
```

### 4. Estructura de Políticas

**Separación de Concerns**:

```cedar
// 1. Políticas de Aislamiento (Tenant/Multi-tenancy)
@id("tenant_isolation")
forbid(principal, action, resource) 
unless { principal.tenant_id == resource.tenant_id };

// 2. Políticas de Permisos (RBAC)
@id("admin_full_access")
permit(principal, action, resource) 
when { principal.role == "admin" };

// 3. Políticas de Propiedad (Ownership)
@id("owner_permissions")
permit(principal, action, resource) 
when { resource.owner_id == principal.id };

// 4. Políticas Específicas de Acción
@id("create_document_admin")
permit(
    principal,
    action == Action::"Create",
    resource
) when { principal.role == "admin" };
```

### 5. Contexto de Autorización

**Incluir información relevante**:

```rust
let context = json!({
    "ip_address": "192.168.1.1",
    "timestamp": "2024-10-17T00:00:00Z",
    "user_agent": "Mozilla/5.0...",
    "mfa_verified": true,
    "session_id": "sess_12345"
});
```

**Usar en políticas**:
```cedar
permit(principal, action, resource)
when {
    context.mfa_verified == true &&
    context.ip_address.isInRange(ip("10.0.0.0/8"))
};
```

### 6. Validación y Testing

**Validar esquema**:
```rust
let schema = Schema::from_json_str(&schema_json)?;
// Cedar valida automáticamente el formato
```

**Validar políticas contra esquema**:
```rust
let validator = Validator::new(schema);
let result = validator.validate(&policy_set, ValidationMode::default());
```

**Tests de políticas**:
```rust
#[test]
fn test_tenant_isolation() {
    let alice = User { tenant_id: "tenant-a", ... };
    let bob_doc = Document { tenant_id: "tenant-b", ... };
    
    let decision = authorizer.is_authorized(
        alice.to_request(Action::Read, bob_doc)
    );
    
    assert_eq!(decision, Decision::Deny);
}
```

### 7. Logging y Auditoría

**NO incluir información sensible en identificadores**:

```rust
// ❌ INCORRECTO
User::"jane.doe@company.com"

// ✅ CORRECTO
User::"user_a1b2c3d4"  // UUID, luego buscar email en BD
```

**Razón**: Los identificadores aparecen en logs de CloudTrail/auditoría.

### 8. Versionado de Políticas

**Incluir versión en ID**:
```cedar
@id("tenant_isolation_v2")  // v2 indica versión
forbid(principal, action, resource) 
unless { 
    principal.tenant_id == resource.tenant_id &&
    resource.archived == false  // Nueva condición en v2
};
```

**Migración**:
1. Crear nueva política con nuevo ID
2. Probar en staging
3. Desactivar política antigua
4. Activar nueva política
5. Eliminar política antigua después de período de gracia

### 9. Jerarquías de Entidades

**Usar memberOfTypes para grupos**:

```json
{
  "entityTypes": {
    "UserGroup": {
      "memberOfTypes": []
    },
    "User": {
      "memberOfTypes": ["UserGroup"]
    }
  }
}
```

**En políticas**:
```cedar
permit(
    principal in UserGroup::"admins",  // Cualquier User en el grupo
    action,
    resource
);
```

### 10. Rendimiento

**Optimizaciones**:

1. **Políticas específicas primero**: Cedar evalúa en orden
2. **Usar índices en entidades**: Mantener entity store optimizado
3. **Cachear PolicySet**: No recargar en cada request
4. **Lazy loading de entidades**: Solo cargar las necesarias

```rust
// ✅ Eficiente - Solo entidades necesarias
let entities = vec![
    principal_entity,
    resource_entity,
    // parent entities si son necesarias
];

// ❌ Ineficiente - Cargar todo el entity store
let entities = load_all_entities_from_db();
```

---

## 🎯 Aplicación a Hodei

### Cambios Recomendados

1. **IDs de Políticas**:
   ```rust
   // Agregar @id al insertar políticas
   let policy_with_id = format!("@id(\"{}\")\n{}", unique_id, policy_content);
   ```

2. **Manejo de Duplicados**:
   ```rust
   // Opción 1: Limpiar al inicio
   policy_set = PolicySet::new();
   
   // Opción 2: Verificar antes de agregar
   if !policy_set.contains_id(&policy_id) {
       policy_set.add(policy)?;
   }
   ```

3. **UUIDs en Entidades**:
   ```rust
   // Ya lo hacemos bien con HRN
   hrn:aws:service:region:tenant:resource/uuid
   ```

4. **Contexto Rico**:
   ```rust
   let context = json!({
       "ip_address": request.ip,
       "tenant_id": tenant_id,
       "timestamp": Utc::now(),
       "mfa_verified": session.mfa_verified
   });
   ```

---

## 📖 Referencias

- [Cedar Policy Language](https://www.cedarpolicy.com/)
- [Cedar JSON Schema Format](https://docs.cedarpolicy.com/schema/json-schema.html)
- [AWS Verified Permissions Best Practices](https://docs.aws.amazon.com/verifiedpermissions/latest/userguide/policies.html)
- [Cedar GitHub](https://github.com/cedar-policy/cedar)

---

**Implementado por**: Hodei Authorization Engine
**Fecha**: 2025-10-17
**Versión Cedar**: 4.7.0
