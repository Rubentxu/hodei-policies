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

    let mut actions: BTreeMap<String, Value> = BTreeMap::new();
    for fragment in inventory::iter::<hodei_provider::ActionSchemaFragment>() {
        let fragment_value: Value = serde_json::from_str(fragment.fragment_json).unwrap();
        let action_name = fragment.name.to_string();
        
        // Si la acción ya existe, combinar resourceTypes
        if let Some(existing) = actions.get_mut(&action_name) {
            if let (Some(existing_resources), Some(new_resources)) = (
                existing.get_mut("appliesTo").and_then(|a| a.get_mut("resourceTypes")),
                fragment_value.get("appliesTo").and_then(|a| a.get("resourceTypes"))
            ) {
                if let (Some(existing_arr), Some(new_arr)) = (existing_resources.as_array_mut(), new_resources.as_array()) {
                    // Agregar nuevos resourceTypes sin duplicar
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