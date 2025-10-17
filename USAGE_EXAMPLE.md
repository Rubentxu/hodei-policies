# 📚 Ejemplo de Uso de hodei-provider

Este ejemplo muestra cómo usar los crates `hodei-provider`, `hodei-kernel` y `hodei-provider-derive` para crear un sistema de autorización con Cedar Policy.

## 🚀 Setup Inicial

### Cargo.toml

```toml
[package]
name = "mi-app-con-hodei"
version = "0.1.0"
edition = "2021"

[dependencies]
hodei-provider = "0.1.0"
cedar-policy = "4.7.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## 📝 Ejemplo Completo

### 1. Definir tus Entidades

```rust
use hodei_provider::{HodeiEntity, hodei_kernel::Hrn};
use serde::{Deserialize, Serialize};

// Entidad User
#[derive(Debug, Clone, Serialize, Deserialize, HodeiEntity)]
#[hodei(entity_type = "MiApp::User")]
pub struct User {
    pub id: Hrn,
    pub role: String,
    pub department: String,
}

// Entidad Document
#[derive(Debug, Clone, Serialize, Deserialize, HodeiEntity)]
#[hodei(entity_type = "MiApp::Document")]
pub struct Document {
    pub id: Hrn,
    
    // Especificar el tipo de entidad para campos Hrn
    #[entity_type = "MiApp::User"]
    pub owner_id: Hrn,
    
    pub title: String,
    pub is_public: bool,
    pub department: String,
}

// Entidad Project
#[derive(Debug, Clone, Serialize, Deserialize, HodeiEntity)]
#[hodei(entity_type = "MiApp::Project")]
pub struct Project {
    pub id: Hrn,
    
    #[entity_type = "MiApp::User"]
    pub manager_id: Hrn,
    
    #[entity_type = "MiApp::User"]
    pub created_by: Hrn,
    
    pub name: String,
    pub budget: i64,
    pub is_active: bool,
}
```

### 2. Definir tus Acciones (Commands)

```rust
use hodei_provider::HodeiAction;

#[derive(Debug, Clone, HodeiAction)]
#[hodei(namespace = "MiApp")]
pub enum DocumentCommand {
    #[hodei(principal = "User", resource = "Document", creates_resource)]
    Create(DocumentCreatePayload),
    
    #[hodei(principal = "User", resource = "Document")]
    Read { id: Hrn },
    
    #[hodei(principal = "User", resource = "Document")]
    Update { id: Hrn, payload: DocumentUpdatePayload },
    
    #[hodei(principal = "User", resource = "Document")]
    Delete { id: Hrn },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCreatePayload {
    pub resource_id: String,
    pub title: String,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUpdatePayload {
    pub title: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, HodeiAction)]
#[hodei(namespace = "MiApp")]
pub enum ProjectCommand {
    #[hodei(principal = "User", resource = "Project", creates_resource)]
    Create(ProjectCreatePayload),
    
    #[hodei(principal = "User", resource = "Project")]
    Read { id: Hrn },
    
    #[hodei(principal = "User", resource = "Project")]
    UpdateBudget { id: Hrn, new_budget: i64 },
    
    #[hodei(principal = "User", resource = "Project")]
    Archive { id: Hrn },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCreatePayload {
    pub resource_id: String,
    pub name: String,
    pub budget: i64,
}
```

### 3. Crear HRNs (Hodei Resource Names)

```rust
use hodei_provider::hodei_kernel::Hrn;

fn main() {
    // Crear HRN para un usuario
    let user_hrn = Hrn::builder()
        .service("users-api")
        .tenant_id("tenant-acme")
        .resource("user/alice")
        .unwrap()
        .build()
        .unwrap();
    
    println!("User HRN: {}", user_hrn);
    // Output: hodei:users-api:global:tenant-acme:user/alice
    
    // Crear HRN para un documento
    let doc_hrn = Hrn::builder()
        .service("docs-api")
        .tenant_id("tenant-acme")
        .resource("document/report-2024")
        .unwrap()
        .build()
        .unwrap();
    
    println!("Document HRN: {}", doc_hrn);
    // Output: hodei:docs-api:global:tenant-acme:document/report-2024
}
```

### 4. Convertir Entidades a Cedar Entities

```rust
use hodei_provider::HodeiEntity;
use cedar_policy::Entity;

fn example_entity_conversion() {
    // Crear un usuario
    let user = User {
        id: Hrn::builder()
            .service("users-api")
            .tenant_id("tenant-acme")
            .resource("user/alice")
            .unwrap()
            .build()
            .unwrap(),
        role: "admin".to_string(),
        department: "engineering".to_string(),
    };
    
    // Convertir a Cedar Entity
    let cedar_entity: Entity = user.to_cedar_entity();
    
    // Ahora puedes usar cedar_entity con el motor de Cedar Policy
    println!("Cedar Entity: {:?}", cedar_entity);
}
```

### 5. Usar con Cedar Policy Engine

```rust
use cedar_policy::{Authorizer, Decision, Entities, PolicySet, Request};
use std::collections::HashSet;

fn check_authorization() {
    // 1. Crear entidades
    let user = User {
        id: Hrn::builder()
            .service("users-api")
            .tenant_id("tenant-acme")
            .resource("user/alice")
            .unwrap()
            .build()
            .unwrap(),
        role: "admin".to_string(),
        department: "engineering".to_string(),
    };
    
    let document = Document {
        id: Hrn::builder()
            .service("docs-api")
            .tenant_id("tenant-acme")
            .resource("document/report-2024")
            .unwrap()
            .build()
            .unwrap(),
        owner_id: user.id.clone(),
        title: "Q4 Report".to_string(),
        is_public: false,
        department: "engineering".to_string(),
    };
    
    // 2. Convertir a Cedar entities
    let mut entities_vec = vec![
        user.to_cedar_entity(),
        document.to_cedar_entity(),
    ];
    
    let entities = Entities::from_entities(
        entities_vec,
        None::<&cedar_policy::Schema>,
    ).unwrap();
    
    // 3. Crear políticas Cedar
    let policy_src = r#"
        // Política: Los admins pueden leer cualquier documento
        permit(
            principal,
            action == Action::"Document::Read",
            resource
        ) when {
            principal.role == "admin"
        };
        
        // Política: Los owners pueden hacer cualquier cosa con sus documentos
        permit(
            principal,
            action,
            resource
        ) when {
            resource has owner_id &&
            resource.owner_id == principal
        };
        
        // Política: Multi-tenancy - solo acceso a recursos del mismo tenant
        forbid(
            principal,
            action,
            resource
        ) unless {
            principal.tenant_id == resource.tenant_id
        };
    "#;
    
    let policy_set = PolicySet::from_str(policy_src).unwrap();
    
    // 4. Crear request de autorización
    let action = DocumentCommand::Read { 
        id: document.id.clone() 
    };
    
    let request = Request::new(
        user.id.to_cedar_entity_uid(),
        action.to_cedar_action_uid(),
        document.id.to_cedar_entity_uid(),
        cedar_policy::Context::empty(),
        None::<&cedar_policy::Schema>,
    ).unwrap();
    
    // 5. Evaluar autorización
    let authorizer = Authorizer::new();
    let response = authorizer.is_authorized(&request, &policy_set, &entities);
    
    match response.decision() {
        Decision::Allow => println!("✅ Autorizado"),
        Decision::Deny => println!("❌ Denegado"),
    }
}
```

### 6. Generación Automática de Esquema Cedar

#### Paso 1: Configurar Cargo.toml

Primero, agrega el feature `schema-discovery` y las dependencias necesarias:

```toml
[package]
name = "mi-app-con-hodei"
version = "0.1.0"
edition = "2021"
build = "build.rs"  # ← Importante: especificar build script

[dependencies]
hodei-provider = "0.1.0"
cedar-policy = "4.7.0"
serde = { version = "1.0", features = ["derive"] }

[build-dependencies]
hodei-provider = "0.1.0"
serde_json = "1.0"

# Crate que contiene tus entidades y acciones
# Debe tener el feature schema-discovery activado
mi-domain = { path = "./mi-domain", features = ["schema-discovery"] }

[features]
schema-discovery = []
```

#### Paso 2: Crear build.rs

Crea un archivo `build.rs` en la raíz de tu proyecto:

```rust
// build.rs
use hodei_provider::inventory;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

// Forzar el enlace del crate que contiene tus entidades
// Sin esto, el linker no incluye el crate y inventory::iter está vacío
#[allow(unused_imports)]
use mi_domain as _;

fn main() {
    println!("cargo:rerun-if-changed=src/");
    
    let namespace = "MiApp";
    
    // Recolectar entidades
    let mut entity_types = BTreeMap::new();
    for fragment in inventory::iter::<hodei_provider::EntitySchemaFragment>() {
        let type_name = fragment.entity_type.split("::").last().unwrap();
        let fragment_value: Value = 
            serde_json::from_str(fragment.fragment_json).unwrap();
        entity_types.insert(type_name.to_string(), fragment_value);
    }
    
    // Recolectar acciones y combinar resourceTypes si tienen el mismo nombre
    let mut actions: BTreeMap<String, Value> = BTreeMap::new();
    for fragment in inventory::iter::<hodei_provider::ActionSchemaFragment>() {
        let fragment_value: Value = 
            serde_json::from_str(fragment.fragment_json).unwrap();
        let action_name = fragment.name.to_string();
        
        // Si la acción ya existe, combinar resourceTypes
        if let Some(existing) = actions.get_mut(&action_name) {
            if let (Some(existing_resources), Some(new_resources)) = (
                existing.get_mut("appliesTo").and_then(|a| a.get_mut("resourceTypes")),
                fragment_value.get("appliesTo").and_then(|a| a.get("resourceTypes"))
            ) {
                if let (Some(existing_arr), Some(new_arr)) = 
                    (existing_resources.as_array_mut(), new_resources.as_array()) {
                    for resource in new_arr {
                        if !existing_arr.contains(resource) {
                            existing_arr.push(resource.clone());
                        }
                    }
                }
            }
        } else {
            actions.insert(action_name, fragment_value);
        }
    }
    
    // Generar esquema completo
    let full_schema = json!({
        namespace: {
            "entityTypes": entity_types,
            "actions": actions
        }
    });
    
    // Guardar a archivo
    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("cedar_schema.json");
    
    fs::write(
        &dest_path,
        serde_json::to_string_pretty(&full_schema).unwrap()
    ).expect("No se pudo escribir cedar_schema.json");
    
    println!("cargo:warning=✅ Esquema Cedar generado en {:?}", dest_path);
}
```

#### Paso 3: Estructurar tu Proyecto

```
mi-app-con-hodei/
├── Cargo.toml
├── build.rs           ← Build script
├── cedar_schema.json  ← Generado automáticamente
├── src/
│   └── main.rs
└── mi-domain/         ← Crate separado con tus entidades
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

#### Paso 4: Configurar el Crate de Dominio

```toml
# mi-domain/Cargo.toml
[package]
name = "mi-domain"
version = "0.1.0"
edition = "2021"

[dependencies]
hodei-provider = "0.1.0"
serde = { version = "1.0", features = ["derive"] }

[features]
schema-discovery = []  # ← Feature para activar generación
```

```rust
// mi-domain/src/lib.rs
use hodei_provider::{HodeiEntity, HodeiAction, hodei_kernel::Hrn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, HodeiEntity)]
#[hodei(entity_type = "MiApp::User")]
pub struct User {
    pub id: Hrn,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, HodeiEntity)]
#[hodei(entity_type = "MiApp::Document")]
pub struct Document {
    pub id: Hrn,
    
    #[entity_type = "MiApp::User"]
    pub owner_id: Hrn,
    
    pub title: String,
}

#[derive(Debug, Clone, HodeiAction)]
#[hodei(namespace = "MiApp")]
pub enum DocumentCommand {
    #[hodei(principal = "User", resource = "Document", creates_resource)]
    Create { title: String },
    
    #[hodei(principal = "User", resource = "Document")]
    Read { id: Hrn },
}
```

#### Paso 5: Compilar y Generar Esquema

```bash
# Compilar con el feature schema-discovery
cargo build --features schema-discovery

# El esquema se genera automáticamente en cedar_schema.json
cat cedar_schema.json
```

#### Esquema Generado (Ejemplo)

```json
{
  "MiApp": {
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
            "tenant_id": {
              "type": "String"
            }
          }
        }
      },
      "Document": {
        "memberOfTypes": [],
        "shape": {
          "type": "Record",
          "attributes": {
            "owner_id": {
              "type": "Entity",
              "name": "MiApp::User",
              "required": true
            },
            "title": {
              "type": "String",
              "required": true
            }
          }
        }
      }
    },
    "actions": {
      "Document::Create": {
        "appliesTo": {
          "principalTypes": ["User"],
          "resourceTypes": ["Document"]
        }
      },
      "Document::Read": {
        "appliesTo": {
          "principalTypes": ["User"],
          "resourceTypes": ["Document"]
        }
      }
    }
  }
}
```

#### 🔄 Regeneración Automática

El esquema se regenera automáticamente cada vez que:
- Compilas el proyecto
- Modificas archivos en `src/`
- Cambias las entidades o acciones

```bash
# Cualquier compilación regenera el esquema
cargo build
cargo run
cargo test
```

## 🎯 Ejemplo Completo de Aplicación

```rust
use hodei_provider::{HodeiEntity, HodeiAction, hodei_kernel::Hrn};
use cedar_policy::{Authorizer, Decision, Entities, PolicySet, Request};

fn main() {
    println!("🚀 Ejemplo de Hodei Provider\n");
    
    // 1. Crear usuarios
    let alice = User {
        id: Hrn::builder()
            .service("users")
            .tenant_id("acme")
            .resource("user/alice")
            .unwrap()
            .build()
            .unwrap(),
        role: "admin".to_string(),
        department: "engineering".to_string(),
    };
    
    let bob = User {
        id: Hrn::builder()
            .service("users")
            .tenant_id("acme")
            .resource("user/bob")
            .unwrap()
            .build()
            .unwrap(),
        role: "user".to_string(),
        department: "sales".to_string(),
    };
    
    // 2. Crear documento
    let doc = Document {
        id: Hrn::builder()
            .service("docs")
            .tenant_id("acme")
            .resource("document/secret-plan")
            .unwrap()
            .build()
            .unwrap(),
        owner_id: alice.id.clone(),
        title: "Secret Plan".to_string(),
        is_public: false,
        department: "engineering".to_string(),
    };
    
    // 3. Configurar Cedar
    let entities = Entities::from_entities(
        vec![
            alice.to_cedar_entity(),
            bob.to_cedar_entity(),
            doc.to_cedar_entity(),
        ],
        None::<&cedar_policy::Schema>,
    ).unwrap();
    
    let policies = PolicySet::from_str(r#"
        permit(principal, action, resource) 
        when { resource has owner_id && resource.owner_id == principal };
        
        forbid(principal, action, resource) 
        unless { principal.tenant_id == resource.tenant_id };
    "#).unwrap();
    
    let authorizer = Authorizer::new();
    
    // 4. Test: Alice lee su documento
    println!("Test 1: Alice lee su propio documento");
    let request = Request::new(
        alice.id.to_cedar_entity_uid(),
        DocumentCommand::Read { id: doc.id.clone() }.to_cedar_action_uid(),
        doc.id.to_cedar_entity_uid(),
        cedar_policy::Context::empty(),
        None::<&cedar_policy::Schema>,
    ).unwrap();
    
    let response = authorizer.is_authorized(&request, &policies, &entities);
    println!("   Resultado: {:?}\n", response.decision());
    
    // 5. Test: Bob intenta leer el documento de Alice
    println!("Test 2: Bob intenta leer documento de Alice");
    let request = Request::new(
        bob.id.to_cedar_entity_uid(),
        DocumentCommand::Read { id: doc.id.clone() }.to_cedar_action_uid(),
        doc.id.to_cedar_entity_uid(),
        cedar_policy::Context::empty(),
        None::<&cedar_policy::Schema>,
    ).unwrap();
    
    let response = authorizer.is_authorized(&request, &policies, &entities);
    println!("   Resultado: {:?}\n", response.decision());
    
    println!("✅ Ejemplo completado");
}
```

## 📊 Salida Esperada

```
🚀 Ejemplo de Hodei Provider

Test 1: Alice lee su propio documento
   Resultado: Allow

Test 2: Bob intenta leer documento de Alice
   Resultado: Deny

✅ Ejemplo completado
```

## 🔗 Referencias

- **Repositorio completo**: https://github.com/Rubentxu/hodei-policies
- **Ejemplo real**: Ver carpeta `hodei_domain/` y `src/` en el repositorio
- **Cedar Policy**: https://www.cedarpolicy.com/
- **Documentación**: https://docs.rs/hodei-provider

## 💡 Tips

1. **Usa `#[entity_type]`** en todos los campos `Hrn` para especificar el tipo de entidad
2. **Multi-tenancy**: Siempre incluye `tenant_id` en tus HRNs
3. **Acciones específicas**: Usa formato `ResourceType::ActionName` para claridad
4. **Políticas**: Empieza con políticas simples y ve agregando complejidad
5. **Tests**: Escribe tests para tus políticas Cedar

¡Disfruta usando Hodei Provider! 🎉
