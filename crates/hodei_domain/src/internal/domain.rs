use serde::{Deserialize, Serialize};
use hodei_authz::HodeiEntity;
use cedar_policy::{Entity, EntityUid};
use hodei_hrn::api::Hrn;

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

    #[entity_type = "HodeiMVP::User"]
    pub owner_id: Hrn,

    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCreatePayload {
    pub resource_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<Hrn>,
    pub is_public: bool,
}

impl DocumentCreatePayload {
    pub(crate) fn to_virtual_entity(&self, context: &RequestContext) -> Entity {
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

// ============================================================================
// Nueva Entidad: Artifact (para demostrar escalabilidad)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone, HodeiEntity, sqlx::FromRow)]
#[hodei(entity_type = "HodeiMVP::Artifact")]
pub struct Artifact {
    pub id: Hrn,

    /// Usuario que creó el artifact
    #[entity_type = "HodeiMVP::User"]
    pub created_by: Hrn,

    /// Usuario que lo modificó por última vez
    #[entity_type = "HodeiMVP::User"]
    pub updated_by: Hrn,

    /// Documento asociado - TODO: hacer opcional cuando el macro soporte Option<Hrn>
    #[entity_type = "HodeiMVP::Document"]
    pub document_id: Hrn,

    pub name: String,
    pub artifact_type: String,
    pub version: String,
    pub is_active: bool,
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
    pub(crate) fn to_virtual_entity(&self, context: &RequestContext) -> Entity {
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
