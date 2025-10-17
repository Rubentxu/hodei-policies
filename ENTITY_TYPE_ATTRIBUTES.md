# Atributos de Tipo de Entidad en Hodei

## 🎯 Problema Resuelto

La librería `hodei_provider_derive` ahora soporta **especificación explícita de tipos de entidad** mediante atributos, haciéndola completamente escalable para cualquier aplicación.

## 📝 Uso Básico

### Opción 1: Inferencia Automática (Conveniente)

Si no especificas atributos, el sistema infiere el tipo basándose en el nombre del campo:

```rust
use hodei_provider::HodeiEntity;
use kernel::domain::value_objects::Hrn;

#[derive(HodeiEntity)]
#[hodei-authz-sdk(entity_type = "MyApp::Document")]
pub struct Document {
    pub id: Hrn,
    pub owner_id: Hrn,        // Inferido como "MyApp::User"
    pub group_id: Hrn,        // Inferido como "MyApp::Group"
    pub created_by: Hrn,      // Inferido como "MyApp::User"
    pub is_public: bool,
}
```

**Reglas de Inferencia:**
- `owner_id`, `user_id`, `created_by`, `updated_by` → `{Namespace}::User`
- `group_id`, `team_id` → `{Namespace}::Group`
- `document_id`, `file_id` → `{Namespace}::Document`
- `organization_id`, `org_id` → `{Namespace}::Organization`
- `role_id` → `{Namespace}::Role`
- Otros: Capitaliza el nombre sin `_id` → `{Namespace}::{CapitalizedName}`

### Opción 2: Especificación Explícita (Recomendado para Producción)

Para máxima claridad y control, especifica el tipo explícitamente:

```rust
use hodei_provider::HodeiEntity;
use kernel::domain::value_objects::Hrn;

#[derive(HodeiEntity)]
#[hodei-authz-sdk(entity_type = "MyApp::Document")]
pub struct Document {
    pub id: Hrn,
    
    #[entity_type = "MyApp::User"]
    pub owner_id: Hrn,
    
    #[entity_type = "MyApp::Organization"]
    pub organization_id: Hrn,
    
    #[entity_type = "CustomNamespace::CustomEntity"]
    pub custom_ref: Hrn,
    
    pub is_public: bool,
}
```

## 🌟 Ejemplos Avanzados

### Aplicación Multi-Tenant

```rust
#[derive(HodeiEntity)]
#[hodei-authz-sdk(entity_type = "SaaS::Workspace")]
pub struct Workspace {
    pub id: Hrn,
    
    #[entity_type = "SaaS::Organization"]
    pub organization_id: Hrn,
    
    #[entity_type = "SaaS::User"]
    pub created_by: Hrn,
    
    pub name: String,
    pub is_active: bool,
}
```

### Sistema de Permisos Complejo

```rust
#[derive(HodeiEntity)]
#[hodei-authz-sdk(entity_type = "IAM::Permission")]
pub struct Permission {
    pub id: Hrn,
    
    #[entity_type = "IAM::Role"]
    pub role_id: Hrn,
    
    #[entity_type = "IAM::Resource"]
    pub resource_id: Hrn,
    
    #[entity_type = "IAM::Action"]
    pub action_id: Hrn,
    
    pub allow: bool,
}
```

### Integración con Sistemas Externos

```rust
#[derive(HodeiEntity)]
#[hodei-authz-sdk(entity_type = "Integration::SyncJob")]
pub struct SyncJob {
    pub id: Hrn,
    
    // Referencia a entidad de otro namespace
    #[entity_type = "ExternalSystem::Account"]
    pub external_account_id: Hrn,
    
    // Referencia a entidad interna
    #[entity_type = "Integration::User"]
    pub user_id: Hrn,
    
    pub status: String,
}
```

## 🔧 Esquema Cedar Generado

### Sin Atributos (Inferencia)

```rust
pub struct Document {
    pub owner_id: Hrn,  // Inferido como "MyApp::User"
}
```

Genera:

```json
{
  "MyApp": {
    "entityTypes": {
      "Document": {
        "shape": {
          "type": "Record",
          "attributes": {
            "owner_id": {
              "type": "Entity",
              "name": "MyApp::User",
              "required": true
            }
          }
        }
      }
    }
  }
}
```

### Con Atributos (Explícito)

```rust
pub struct Document {
    #[entity_type = "CustomNS::CustomUser"]
    pub owner_id: Hrn,
}
```

Genera:

```json
{
  "MyApp": {
    "entityTypes": {
      "Document": {
        "shape": {
          "type": "Record",
          "attributes": {
            "owner_id": {
              "type": "Entity",
              "name": "CustomNS::CustomUser",
              "required": true
            }
          }
        }
      }
    }
  }
}
```

## ✅ Ventajas de Este Enfoque

1. **Escalable**: Cualquier aplicación puede definir sus propios tipos
2. **Flexible**: Soporta múltiples namespaces en la misma entidad
3. **Conveniente**: Inferencia automática para casos comunes
4. **Explícito**: Atributos para casos especiales
5. **Type-Safe**: Validación en tiempo de compilación
6. **Documentado**: El código es auto-documentado

## 🚀 Migración desde Versión Anterior

Si tenías código que dependía del hardcoding anterior:

```rust
// ANTES (hardcoded)
pub struct Document {
    pub owner_id: Hrn,  // Siempre era "HodeiMVP::User"
}

// DESPUÉS (inferido con tu namespace)
#[hodei-authz-sdk(entity_type = "MiApp::Document")]
pub struct Document {
    pub owner_id: Hrn,  // Ahora es "MiApp::User"
}

// O EXPLÍCITO (control total)
#[hodei-authz-sdk(entity_type = "MiApp::Document")]
pub struct Document {
    #[entity_type = "MiApp::User"]
    pub owner_id: Hrn,
}
```

## 📚 Referencias

- [Cedar Policy Language](https://www.cedarpolicy.com/)
- [Entity Types en Cedar](https://docs.cedarpolicy.com/schema/schema.html#entity-types)
- [Rust Procedural Macros](https://doc.rust-lang.org/reference/procedural-macros.html)

---

**Versión**: 2.0.0  
**Fecha**: 2025-10-17  
**Estado**: ✅ Implementado y Probado
