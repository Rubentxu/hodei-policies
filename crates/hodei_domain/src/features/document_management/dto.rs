use serde::{Deserialize, Serialize};
use hodei_authz::HodeiAction;
use cedar_policy::{Entity, EntityUid};
use hodei_hrn::api::Hrn;

// ============================================================================
// Comandos para Document
// ============================================================================

#[derive(Debug, Clone, HodeiAction)]
#[hodei(namespace = "HodeiMVP")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Hrn>,
    pub is_public: bool,
}

impl DocumentCreatePayload {
    pub fn to_virtual_entity(&self, context: &crate::internal::domain::RequestContext) -> Entity {
        let hrn = Hrn::builder()
            .service("documents-api")
            .tenant_id(&context.tenant_id)
            .resource(&format!("document/{}", self.resource_id)).unwrap()
            .build().unwrap();

        let euid = EntityUid::from_type_name_and_id(
            "HodeiMVP::Document".parse().unwrap(),
            hrn.to_string().parse().unwrap(),
        );

        let mut attrs = std::collections::HashMap::new();

        // owner_id debe ser un EntityUid para que Cedar pueda compararlo con principal
        if let Some(owner_hrn) = &self.owner_id {
            let owner_euid = EntityUid::from_type_name_and_id(
                "HodeiMVP::User".parse().unwrap(),
                owner_hrn.to_string().parse().unwrap(),
            );
            attrs.insert("owner_id".into(), cedar_policy::RestrictedExpression::new_entity_uid(owner_euid));
        }

        attrs.insert("is_public".into(), cedar_policy::RestrictedExpression::new_bool(self.is_public));
        attrs.insert("tenant_id".into(), cedar_policy::RestrictedExpression::new_string(hrn.tenant_id.clone()));
        attrs.insert("service".into(), cedar_policy::RestrictedExpression::new_string(hrn.service.clone()));

        Entity::new(euid, attrs, std::collections::HashSet::new()).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentUpdatePayload {
    pub is_public: Option<bool>,
}
