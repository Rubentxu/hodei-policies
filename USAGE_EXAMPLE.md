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

Si usas el feature `schema-discovery`, puedes generar el esquema automáticamente:

```rust
// build.rs
use hodei_provider::inventory;
use serde_json::json;
use std::collections::BTreeMap;

fn main() {
    let namespace = "MiApp";
    
    // Recolectar entidades
    let mut entity_types = BTreeMap::new();
    for fragment in inventory::iter::<hodei_provider::EntitySchemaFragment>() {
        let type_name = fragment.entity_type.split("::").last().unwrap();
        let fragment_value: serde_json::Value = 
            serde_json::from_str(fragment.fragment_json).unwrap();
        entity_types.insert(type_name.to_string(), fragment_value);
    }
    
    // Recolectar acciones
    let mut actions = BTreeMap::new();
    for fragment in inventory::iter::<hodei_provider::ActionSchemaFragment>() {
        let fragment_value: serde_json::Value = 
            serde_json::from_str(fragment.fragment_json).unwrap();
        actions.insert(fragment.name.to_string(), fragment_value);
    }
    
    // Generar esquema completo
    let full_schema = json!({
        namespace: {
            "entityTypes": entity_types,
            "actions": actions
        }
    });
    
    // Guardar a archivo
    std::fs::write(
        "cedar_schema.json",
        serde_json::to_string_pretty(&full_schema).unwrap()
    ).unwrap();
    
    println!("✅ Esquema Cedar generado");
}
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
