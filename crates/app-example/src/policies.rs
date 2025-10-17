//! Cedar policies for the document application

/// Get all Cedar policies for the application
pub fn get_policies() -> Vec<&'static str> {
    vec![
        // Policy 1: Document owners can do anything with their documents
        r#"
permit(
    principal,
    action,
    resource
) when {
    resource has owner_id &&
    resource.owner_id == principal
};
"#,
        // Policy 2: Public documents can be read by anyone
        r#"
permit(
    principal,
    action == Action::"Document::Read",
    resource
) when {
    resource has is_public &&
    resource.is_public == true
};
"#,
        // Policy 3: Admins can do anything
        r#"
permit(
    principal,
    action,
    resource
) when {
    principal has role &&
    principal.role == "admin"
};
"#,
        // Policy 4: Editors can read and update documents
        r#"
permit(
    principal,
    action in [Action::"Document::Read", Action::"Document::Update"],
    resource
) when {
    principal has role &&
    principal.role == "editor"
};
"#,
        // Policy 5: Viewers can only read documents
        r#"
permit(
    principal,
    action == Action::"Document::Read",
    resource
) when {
    principal has role &&
    principal.role == "viewer"
};
"#,
        // Policy 6: Users can view their own profile
        r#"
permit(
    principal,
    action == Action::"User::ViewProfile",
    resource
) when {
    principal == resource
};
"#,
        // Policy 7: Users can update their own profile
        r#"
permit(
    principal,
    action == Action::"User::UpdateProfile",
    resource
) when {
    principal == resource
};
"#,
        // Policy 8: Only admins can change user roles
        r#"
permit(
    principal,
    action == Action::"User::ChangeRole",
    resource
) when {
    principal has role &&
    principal.role == "admin"
};
"#,
    ]
}

/// Get the Cedar schema for the application
pub fn get_schema() -> &'static str {
    r#"
{
    "DocApp": {
        "entityTypes": {
            "User": {
                "shape": {
                    "type": "Record",
                    "attributes": {
                        "email": { "type": "String" },
                        "name": { "type": "String" },
                        "role": { "type": "String" }
                    }
                }
            },
            "Document": {
                "shape": {
                    "type": "Record",
                    "attributes": {
                        "owner_id": { "type": "Entity", "name": "User" },
                        "title": { "type": "String" },
                        "content": { "type": "String" },
                        "is_public": { "type": "Boolean" }
                    }
                }
            }
        },
        "actions": {
            "Document::Read": {
                "appliesTo": {
                    "principalTypes": ["User"],
                    "resourceTypes": ["Document"]
                }
            },
            "Document::Create": {
                "appliesTo": {
                    "principalTypes": ["User"],
                    "resourceTypes": ["Document"]
                }
            },
            "Document::Update": {
                "appliesTo": {
                    "principalTypes": ["User"],
                    "resourceTypes": ["Document"]
                }
            },
            "Document::Delete": {
                "appliesTo": {
                    "principalTypes": ["User"],
                    "resourceTypes": ["Document"]
                }
            },
            "User::ViewProfile": {
                "appliesTo": {
                    "principalTypes": ["User"],
                    "resourceTypes": ["User"]
                }
            },
            "User::UpdateProfile": {
                "appliesTo": {
                    "principalTypes": ["User"],
                    "resourceTypes": ["User"]
                }
            },
            "User::ChangeRole": {
                "appliesTo": {
                    "principalTypes": ["User"],
                    "resourceTypes": ["User"]
                }
            }
        }
    }
}
"#
}
