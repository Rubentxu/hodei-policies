use hodei_provider::{self, inventory};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;

// Forzar el enlace del crate hodei_domain para que inventory pueda recolectar sus items
// Sin esto, el linker no incluye el crate y inventory::iter está vacío
#[allow(unused_imports)]
use hodei_domain as _;

fn main() {
    println!("cargo:rerun-if-changed=src/");

    let namespace = "HodeiMVP";

    let mut entity_types = BTreeMap::new();
    for fragment in inventory::iter::<hodei_provider::EntitySchemaFragment>() {
        let type_name = fragment.entity_type.split("::").last().unwrap();
        let fragment_value: Value = serde_json::from_str(fragment.fragment_json).unwrap();
        entity_types.insert(type_name.to_string(), fragment_value);
    }

    let mut actions = BTreeMap::new();
    for fragment in inventory::iter::<hodei_provider::ActionSchemaFragment>() {
        let fragment_value: Value = serde_json::from_str(fragment.fragment_json).unwrap();
        actions.insert(fragment.name.to_string(), fragment_value);
    }

    let full_schema = json!({
        namespace: {
            "entityTypes": entity_types,
            "actions": actions
        }
    });

    let out_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("cedar_schema.json");
    let schema_str = serde_json::to_string_pretty(&full_schema).unwrap();

    fs::write(&dest_path, schema_str).expect("No se pudo escribir cedar_schema.json");

    println!("cargo:warning=✅ Esquema Cedar generado en {:?}", dest_path);
}