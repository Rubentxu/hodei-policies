use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Lit};

#[proc_macro_derive(HodeiEntity, attributes(hodei))]
pub fn hodei_entity_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let mut entity_type_str: Option<String> = None;
    for attr in &ast.attrs {
        if attr.path().is_ident("hodei") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("entity_type") {
                    if let Ok(Lit::Str(s)) = meta.value()?.parse() {
                        entity_type_str = Some(s.value());
                    }
                }
                Ok(())
            });
        }
    }
    let entity_type_str = entity_type_str.expect("#[derive(HodeiEntity)] requiere #[hodei(entity_type = \"...\")]");

    let mut attributes = serde_json::Map::new();
    let mut attr_map_assignments: Vec<proc_macro2::TokenStream> = Vec::new();

    if let Data::Struct(data_struct) = &ast.data {
        if let Fields::Named(fields) = &data_struct.fields {
            for field in &fields.named {
                let field_ident = field.ident.as_ref().unwrap();
                let field_name = field_ident.to_string();
                if field_name == "id" { continue; }
                
                let type_ident = match &field.ty {
                    syn::Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_else(|| "Complex".to_string()),
                    _ => "Complex".to_string(),
                };
                
                let cedar_type = match type_ident.as_str() {
                    "String" => "String",
                    "i64" | "u64" | "i32" | "u32" | "usize" => "Long",
                    "bool" => "Boolean",
                    _ => "String",
                };
                
                attributes.insert(field_name.clone(), serde_json::json!({ "type": cedar_type, "required": true }));
                
                let value_expr = if type_ident == "Hrn" {
                    quote! { cedar_policy::RestrictedExpression::new_string(self.#field_ident.to_string()) }
                } else if type_ident == "String" {
                    quote! { cedar_policy::RestrictedExpression::new_string(self.#field_ident.clone()) }
                } else if type_ident == "bool" {
                    quote! { cedar_policy::RestrictedExpression::new_bool(self.#field_ident) }
                } else {
                    quote! { cedar_policy::RestrictedExpression::new_long(self.#field_ident as i64) }
                };
                
                attr_map_assignments.push(quote! {
                    attrs.insert(#field_name.into(), #value_expr);
                });
            }
        }
    }

    attributes.insert("tenant_id".to_string(), serde_json::json!({ "type": "String" }));
    attributes.insert("service".to_string(), serde_json::json!({ "type": "String" }));
    let schema_fragment_json = serde_json::json!({ "memberOfTypes": [], "attributes": attributes });
    let schema_fragment_str = serde_json::to_string(&schema_fragment_json).unwrap();

    let expanded = quote! {
        impl hodei_provider::RuntimeHodeiEntityMapper for #struct_name {
            fn hodei_type_name(&self) -> &'static str { #entity_type_str }
            fn hodei_id(&self) -> String { self.id.resource_id.clone() }
            fn hodei_hrn(&self) -> &kernel::Hrn { &self.id }
            fn to_cedar_entity(&self) -> cedar_policy::Entity {
                let euid = self.to_cedar_euid();
                let mut attrs = std::collections::HashMap::new();
                #(#attr_map_assignments)*
                attrs.insert("tenant_id".into(), cedar_policy::RestrictedExpression::new_string(self.id.tenant_id.clone()));
                attrs.insert("service".into(), cedar_policy::RestrictedExpression::new_string(self.id.service.clone()));
                cedar_policy::Entity::new(euid, attrs, std::collections::HashSet::new()).unwrap()
            }
        }
        #[cfg(feature = "schema-discovery")]
        hodei_provider::inventory::submit! {
            hodei_provider::EntitySchemaFragment { entity_type: #entity_type_str, fragment_json: #schema_fragment_str, }
        }
    };
    expanded.into()
}

#[proc_macro_derive(HodeiAction, attributes(hodei))]
pub fn hodei_action_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let enum_name = &ast.ident;

    let mut namespace: Option<String> = None;
    for attr in &ast.attrs {
        if attr.path().is_ident("hodei") {
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("namespace") {
                    if let Ok(Lit::Str(s)) = meta.value()?.parse() {
                        namespace = Some(s.value());
                    }
                }
                Ok(())
            });
        }
    }
    let _namespace = namespace.expect("#[derive(HodeiAction)] requiere #[hodei(namespace = \"...\")]");

    let data_enum = match &ast.data { 
        Data::Enum(de) => de, 
        _ => panic!("HodeiAction solo se puede derivar en enums"), 
    };
    
    let mut inventory_submissions: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut euid_match_arms: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut creates_resource_match_arms: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut virtual_entity_match_arms: Vec<proc_macro2::TokenStream> = Vec::new();

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        let action_name_str = variant_name.to_string();
        let action_euid_str = format!("Action::\"{}\"", action_name_str);
        let mut principal_types: Vec<String> = Vec::new();
        let mut resource_types: Vec<String> = Vec::new();
        let mut is_create_action = false;

        for attr in &variant.attrs {
            if attr.path().is_ident("hodei") {
                let _ = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("principal") {
                        if let Ok(Lit::Str(s)) = meta.value()?.parse() {
                            principal_types.push(s.value());
                        }
                    } else if meta.path.is_ident("resource") {
                        if let Ok(Lit::Str(s)) = meta.value()?.parse() {
                            resource_types.push(s.value());
                        }
                    } else if meta.path.is_ident("creates_resource") {
                        is_create_action = true;
                    }
                    Ok(())
                });
            }
        }

        let action_schema_json = serde_json::json!({ 
            "appliesTo": { 
                "principalTypes": principal_types, 
                "resourceTypes": resource_types 
            } 
        });
        let action_schema_str = serde_json::to_string(&action_schema_json).unwrap();
        
        inventory_submissions.push(quote! {
            #[cfg(feature = "schema-discovery")]
            hodei_provider::inventory::submit! { 
                hodei_provider::ActionSchemaFragment { 
                    name: #action_name_str, 
                    fragment_json: #action_schema_str 
                } 
            }
        });

        let fields_pattern = match &variant.fields { 
            Fields::Named(_) => quote! { {..} }, 
            Fields::Unnamed(_) => quote! { (..) }, 
            Fields::Unit => quote! {}, 
        };
        
        euid_match_arms.push(quote! { Self::#variant_name #fields_pattern => #action_euid_str.parse().unwrap() });
        
        if is_create_action {
            creates_resource_match_arms.push(quote! { Self::#variant_name #fields_pattern => true });
            match &variant.fields {
                Fields::Unnamed(f) if f.unnamed.len() == 1 => {
                    virtual_entity_match_arms.push(quote! { 
                        Self::#variant_name(payload) => {
                            let ctx = context.downcast_ref::<crate::RequestContext>().expect("Invalid context type");
                            Some(payload.to_virtual_entity(ctx))
                        }
                    });
                }
                _ => {
                    virtual_entity_match_arms.push(quote! { Self::#variant_name #fields_pattern => None });
                }
            }
        } else {
            creates_resource_match_arms.push(quote! { Self::#variant_name #fields_pattern => false });
        }
    }
    
    virtual_entity_match_arms.push(quote! { _ => None });

    let expanded = quote! {
        #(#inventory_submissions)*
        impl hodei_provider::RuntimeHodeiActionMapper for #enum_name {
            fn to_cedar_action_euid(&self) -> cedar_policy::EntityUid { 
                match self { #(#euid_match_arms,)* } 
            }
            fn creates_resource_from_payload(&self) -> bool { 
                match self { #(#creates_resource_match_arms,)* } 
            }
            fn get_payload_as_virtual_entity(&self, context: &dyn std::any::Any) -> Option<cedar_policy::Entity> { 
                match self { #(#virtual_entity_match_arms,)* } 
            }
        }
    };
    expanded.into()
}
