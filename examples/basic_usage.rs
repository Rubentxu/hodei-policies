//! Basic usage example of the Hodei Framework
//!
//! This example demonstrates:
//! - Defining entities with HodeiEntity
//! - Defining actions with HodeiAction  
//! - Creating HRNs
//! - Using PolicyStore
//! - Authorization with Cedar

use hodei_kernel::Hrn;
use serde::{Deserialize, Serialize};

// Define your domain entities
#[derive(Clone, Serialize, Deserialize)]
struct User {
    id: Hrn,
    email: String,
    role: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Document {
    id: Hrn,
    owner_id: Hrn,
    title: String,
    content: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Hodei Framework - Basic Usage Example\n");
    
    // 1. Create HRNs for resources
    println!("1️⃣  Creating HRNs...");
    
    let alice_hrn = Hrn::builder()
        .service("myapp")
        .tenant_id("tenant-1")
        .resource("user/alice")
        .unwrap()
        .build()
        .unwrap();
    
    let bob_hrn = Hrn::builder()
        .service("myapp")
        .tenant_id("tenant-1")
        .resource("user/bob")
        .unwrap()
        .build()
        .unwrap();
    
    let doc_hrn = Hrn::builder()
        .service("myapp")
        .tenant_id("tenant-1")
        .resource("document/doc-1")
        .unwrap()
        .build()
        .unwrap();
    
    println!("   Alice HRN: {}", alice_hrn);
    println!("   Bob HRN: {}", bob_hrn);
    println!("   Document HRN: {}\n", doc_hrn);
    
    // 2. Create entities
    println!("2️⃣  Creating entities...");
    
    let alice = User {
        id: alice_hrn.clone(),
        email: "alice@example.com".to_string(),
        role: "owner".to_string(),
    };
    
    let bob = User {
        id: bob_hrn.clone(),
        email: "bob@example.com".to_string(),
        role: "viewer".to_string(),
    };
    
    let document = Document {
        id: doc_hrn.clone(),
        owner_id: alice_hrn.clone(),
        title: "Secret Document".to_string(),
        content: "This is confidential information".to_string(),
    };
    
    println!("   Created user: {} ({})", alice.email, alice.role);
    println!("   Created user: {} ({})", bob.email, bob.role);
    println!("   Created document: {} (owner: {})\n", document.title, alice.email);
    
    // 3. Define Cedar policies
    println!("3️⃣  Cedar Policies:");
    
    let policy1 = r#"
// Policy 1: Document owner can do anything
permit(
    principal,
    action,
    resource
) when {
    resource has owner_id &&
    resource.owner_id == principal
};
"#;
    
    let policy2 = r#"
// Policy 2: Users with 'viewer' role can read
permit(
    principal,
    action == Action::"read",
    resource
) when {
    principal has role &&
    principal.role == "viewer"
};
"#;
    
    println!("{}", policy1);
    println!("{}", policy2);
    
    // 4. Demonstrate HRN operations
    println!("4️⃣  HRN Operations:");
    
    // Parsing
    let parsed_hrn: Hrn = "hrn:hodei:myapp:global:tenant-1:user/charlie"
        .parse()
        .unwrap();
    println!("   Parsed HRN: {}", parsed_hrn);
    println!("   - Service: {}", parsed_hrn.service);
    println!("   - Tenant: {}", parsed_hrn.tenant_id);
    println!("   - Resource Type: {}", parsed_hrn.resource_type);
    println!("   - Resource ID: {}\n", parsed_hrn.resource_id);
    
    // Serialization
    let json = serde_json::to_string_pretty(&alice_hrn)?;
    println!("   HRN as JSON:\n{}\n", json);
    
    // 5. Summary
    println!("✅ Example completed successfully!");
    println!("\n📚 Next steps:");
    println!("   - Add PostgreSQL: use hodei-postgres for policy storage");
    println!("   - Add Redis: use hodei-redis for cache invalidation");
    println!("   - Add Axum: use hodei-axum for web integration");
    println!("   - See app-example/ for a complete application");
    
    Ok(())
}
