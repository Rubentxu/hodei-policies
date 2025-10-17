//! Authorization service using Hodei SDK

use crate::domain::{Document, User};
use crate::policies;
use cedar_policy::{Authorizer, Decision, Entities, EntityUid, PolicySet, Request, Schema};
use hodei_authz::{CacheInvalidation, PolicyStore};
use hodei_authz_postgres::PostgresPolicyStore;
use hodei_authz_redis::RedisCacheInvalidation;
use hodei_hrn::Hrn;
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Policy store error: {0}")]
    PolicyStore(String),
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Cedar error: {0}")]
    Cedar(String),
    #[error("Schema error: {0}")]
    Schema(#[from] cedar_policy::SchemaError),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized")]
    Unauthorized,
}

/// Authorization service for the document application
pub struct AuthService {
    policy_store: Arc<PostgresPolicyStore>,
    cache_invalidation: Arc<RedisCacheInvalidation>,
    authorizer: Authorizer,
    schema: Arc<Schema>,
    policy_set: Arc<RwLock<PolicySet>>,
}

impl AuthService {
    /// Create a new authorization service
    pub async fn new(pg_pool: PgPool, redis_url: &str) -> Result<Self, ServiceError> {
        // Setup policy store
        let policy_store = PostgresPolicyStore::new(pg_pool);
        policy_store
            .migrate()
            .await
            .map_err(|e| ServiceError::PolicyStore(e.to_string()))?;
        
        // Setup cache invalidation
        let cache_invalidation = RedisCacheInvalidation::new(redis_url)
            .await
            .map_err(|e| ServiceError::Cache(e.to_string()))?;
        
        // Load schema
        let schema = Schema::from_json_str(policies::get_schema())?;
        
        // Initialize policies
        let policy_set = Self::load_initial_policies(&policy_store).await?;
        
        let authorizer = Authorizer::new();
        
        Ok(Self {
            policy_store: Arc::new(policy_store),
            cache_invalidation: Arc::new(cache_invalidation),
            authorizer,
            schema: Arc::new(schema),
            policy_set: Arc::new(RwLock::new(policy_set)),
        })
    }
    
    /// Load initial policies from code
    async fn load_initial_policies(
        store: &PostgresPolicyStore,
    ) -> Result<PolicySet, ServiceError> {
        // Check if policies exist
        let existing = store
            .list_policies()
            .await
            .map_err(|e| ServiceError::PolicyStore(e.to_string()))?;
        
        // If no policies, load from code
        if existing.is_empty() {
            tracing::info!("Loading initial policies...");
            for policy_content in policies::get_policies() {
                store
                    .create_policy(policy_content.to_string())
                    .await
                    .map_err(|e| ServiceError::PolicyStore(e.to_string()))?;
            }
        }
        
        // Load all policies
        store
            .load_all_policies()
            .await
            .map_err(|e| ServiceError::PolicyStore(e.to_string()))
    }
    
    /// Check if a user can perform an action on a resource
    pub async fn authorize(
        &self,
        principal: &User,
        action: &str,
        resource: &Document,
    ) -> Result<bool, ServiceError> {
        // Convert to Cedar entities
        // Principal and Resource use full HRN format
        let principal_uid: EntityUid = format!("DocApp::User::\"{}\"", principal.id).parse()
            .map_err(|e: cedar_policy::ParseErrors| ServiceError::Cedar(e.to_string()))?;
        
        // Action: if it already contains "::", use as-is, otherwise add DocApp::Action::
        let action_str = if action.contains("::") {
            action.to_string()
        } else {
            format!("DocApp::Action::\"{}\"", action)
        };
        let action_uid: EntityUid = action_str.parse()
            .map_err(|e: cedar_policy::ParseErrors| ServiceError::Cedar(e.to_string()))?;
        
        let resource_uid: EntityUid = format!("DocApp::Document::\"{}\"", resource.id).parse()
            .map_err(|e: cedar_policy::ParseErrors| ServiceError::Cedar(e.to_string()))?;
        
        // Create request
        let request = Request::new(
            principal_uid,
            action_uid,
            resource_uid,
            cedar_policy::Context::empty(),
            Some(&self.schema),
        ).map_err(|e| ServiceError::Cedar(e.to_string()))?;
        
        // Create entities
        let entities = self.create_entities(principal, resource)?;
        
        // Check authorization
        let policy_set = self.policy_set.read().await;
        let response = self.authorizer.is_authorized(&request, &*policy_set, &entities);
        
        Ok(response.decision() == Decision::Allow)
    }
    
    /// Create Cedar entities from domain objects
    fn create_entities(&self, principal: &User, resource: &Document) -> Result<Entities, ServiceError> {
        // This is simplified - in production you'd use the HodeiEntity trait
        // to automatically convert domain objects to Cedar entities
        
        let entities_json = serde_json::json!([
            {
                "uid": { "type": "DocApp::User", "id": principal.id.to_string() },
                "attrs": {
                    "email": &principal.email,
                    "name": &principal.name,
                    "role": format!("{:?}", principal.role).to_lowercase(),
                },
                "parents": []
            },
            {
                "uid": { "type": "DocApp::Document", "id": resource.id.to_string() },
                "attrs": {
                    "owner_id": { "type": "DocApp::User", "id": resource.owner_id.to_string() },
                    "title": &resource.title,
                    "content": &resource.content,
                    "is_public": resource.is_public,
                },
                "parents": []
            }
        ]);
        
        Entities::from_json_str(&entities_json.to_string(), Some(&self.schema))
            .map_err(|e| ServiceError::Cedar(e.to_string()))
    }
    
    /// Reload policies (called after policy updates)
    pub async fn reload_policies(&self) -> Result<(), ServiceError> {
        let new_policy_set = self
            .policy_store
            .load_all_policies()
            .await
            .map_err(|e| ServiceError::PolicyStore(e.to_string()))?;
        
        let mut policy_set = self.policy_set.write().await;
        *policy_set = new_policy_set;
        
        Ok(())
    }
    
    /// Invalidate cache (notify other instances)
    pub async fn invalidate_cache(&self) -> Result<(), ServiceError> {
        self.cache_invalidation
            .invalidate_policies()
            .await
            .map_err(|e| ServiceError::Cache(e.to_string()))
    }
}
