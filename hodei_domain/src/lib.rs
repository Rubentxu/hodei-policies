use serde::{Deserialize, Serialize};
use hodei_provider::{HodeiEntity, HodeiAction};
use cedar_policy::{Entity, EntityUid};
use kernel::Hrn;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub ip_address: String,
    pub tenant_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, HodeiEntity, sqlx::FromRow)]
#[hodei(entity_type = "HodeiMVP::User")]
pub struct User {
    pub id: Hrn,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, HodeiEntity, sqlx::FromRow)]
#[hodei(entity_type = "HodeiMVP::Document")]
pub struct Document {
    pub id: Hrn,
    pub owner_id: Hrn,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCreatePayload {
    pub resource_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Hrn>,
    pub is_public: bool,
}

impl DocumentCreatePayload {
    pub fn to_virtual_entity(&self, context: &RequestContext) -> Entity {
        let hrn = Hrn::builder()
            .service("documents-api")
            .tenant_id(&context.tenant_id)
            .resource(&format!("document/{}", self.resource_id)).unwrap()
            .build().unwrap();

        let euid = EntityUid::from_type_name_and_id(
            "HodeiMVP::Document".parse().unwrap(),
            hrn.to_string().parse().unwrap(),
        );

        let owner_id_str = self.owner_id.as_ref()
            .map(|h| h.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut attrs = std::collections::HashMap::new();
        attrs.insert("owner_id".into(), cedar_policy::RestrictedExpression::new_string(owner_id_str));
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

#[cfg(feature = "schema-discovery")]
#[used]
pub static HODEI_DOMAIN_INVENTORY_ANCHOR: fn() = || {};
