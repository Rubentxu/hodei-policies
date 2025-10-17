use serde::{Deserialize, Serialize};
use hodei_authz::HodeiAction;
use cedar_policy::{Entity, EntityUid};
use hodei_hrn::api::Hrn;

// ============================================================================
// Comandos para Artifact
// ============================================================================

#[derive(Debug, Clone, HodeiAction)]
#[hodei(namespace = "HodeiMVP")]
pub enum ArtifactCommand {
    #[hodei(principal = "User", resource = "Artifact", creates_resource)]
    Create(ArtifactCreatePayload),
    #[hodei(principal = "User", resource = "Artifact")]
    Read { id: Hrn },
    #[hodei(principal = "User", resource = "Artifact")]
    Update { id: Hrn, payload: ArtifactUpdatePayload },
    #[hodei(principal = "User", resource = "Artifact")]
    Delete { id: Hrn },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactCreatePayload {
    pub resource_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document_id: Option<Hrn>,
    pub name: String,
    pub artifact_type: String,
    pub version: String,
}

impl ArtifactCreatePayload {
    pub fn to_virtual_entity(&self, context: &crate::internal::domain::RequestContext) -> Entity {
        let hrn = Hrn::builder()
            .service("artifacts-api")
            .tenant_id(&context.tenant_id)
            .resource(&format!("artifact/{}", self.resource_id)).unwrap()
            .build().unwrap();

        let euid = EntityUid::from_type_name_and_id(
            "HodeiMVP::Artifact".parse().unwrap(),
            hrn.to_string().parse().unwrap(),
        );

        let mut attrs = std::collections::HashMap::new();

        // created_by y updated_by - en producción vendrían del contexto de autenticación
        // Por ahora usamos un placeholder para demostrar el esquema
        let placeholder_user_hrn = Hrn::builder()
            .service("users-api")
            .tenant_id(&context.tenant_id)
            .resource("user/placeholder").unwrap()
            .build().unwrap();

        let creator_euid = EntityUid::from_type_name_and_id(
                "HodeiMVP::User".parse().unwrap(),
                placeholder_user_hrn.to_string().parse().unwrap(),
            );
        attrs.insert("created_by".into(), cedar_policy::RestrictedExpression::new_entity_uid(creator_euid.clone()));
        attrs.insert("updated_by".into(), cedar_policy::RestrictedExpression::new_entity_uid(creator_euid));

        // document_id opcional
        if let Some(doc_hrn) = &self.document_id {
            let doc_euid = EntityUid::from_type_name_and_id(
                "HodeiMVP::Document".parse().unwrap(),
                doc_hrn.to_string().parse().unwrap(),
            );
            attrs.insert("document_id".into(), cedar_policy::RestrictedExpression::new_entity_uid(doc_euid));
        }

        attrs.insert("name".into(), cedar_policy::RestrictedExpression::new_string(self.name.clone()));
        attrs.insert("artifact_type".into(), cedar_policy::RestrictedExpression::new_string(self.artifact_type.clone()));
        attrs.insert("version".into(), cedar_policy::RestrictedExpression::new_string(self.version.clone()));
        attrs.insert("is_active".into(), cedar_policy::RestrictedExpression::new_bool(true));
        attrs.insert("tenant_id".into(), cedar_policy::RestrictedExpression::new_string(context.tenant_id.clone()));
        attrs.insert("service".into(), cedar_policy::RestrictedExpression::new_string("artifacts-api".to_string()));

        Entity::new(euid, attrs, std::collections::HashSet::new()).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactUpdatePayload {
    pub name: Option<String>,
    pub version: Option<String>,
    pub is_active: Option<bool>,
}
