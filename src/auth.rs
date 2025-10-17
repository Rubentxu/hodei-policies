use cedar_policy::{Authorizer, Decision, Entities, Policy, PolicyId, PolicySet, Request, Schema};
use std::sync::Arc;
use tokio::sync::RwLock;
use thiserror::Error;
use async_trait::async_trait;
use sqlx::{PgPool, Row};

#[derive(Debug, Error)]
pub enum AuthServiceError {
    #[error("DB Error: {0}")] Db(#[from] sqlx::Error),
    #[error("I/O Error: {0}")] Io(#[from] std::io::Error),
    #[error("Schema Error: {0}")] Schema(#[from] cedar_policy::SchemaError),
    #[error("Policy Parse Error: {0}")] PolicyParse(cedar_policy::ParseErrors),
    #[error("Policy Add Error: {0}")] PolicyAdd(#[from] cedar_policy::PolicySetError),
    #[error("Policy ID not found for removal")] PolicyIdNotFound,
}

#[async_trait]
pub trait PolicyAdapter: Send + Sync {
    async fn load_policies(&self) -> Result<PolicySet, AuthServiceError>;
    async fn create_policy(&self, content: String) -> Result<String, AuthServiceError>;
    async fn update_policy(&self, id: String, content: String) -> Result<(), AuthServiceError>;
    async fn delete_policy(&self, id: String) -> Result<(), AuthServiceError>;
    async fn get_policy(&self, id: String) -> Result<Option<String>, AuthServiceError>;
    async fn list_policies(&self) -> Result<Vec<(String, String)>, AuthServiceError>;
}

pub struct PostgresAdapter {
    db: PgPool,
}

impl PostgresAdapter {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PolicyAdapter for PostgresAdapter {
    async fn load_policies(&self) -> Result<PolicySet, AuthServiceError> {
        let records = sqlx::query("SELECT id, content FROM policies")
            .fetch_all(&self.db)
            .await?;
        let policy_count = records.len();
        
        // Parsear todas las políticas con IDs explícitos
        let mut policies = Vec::new();
        for record in records {
            let db_id: String = record.try_get("id")?;
            let content: String = record.try_get("content")?;
            
            // Usar Policy::parse con ID explícito (evita "policy0" automático)
            let policy_id = cedar_policy::PolicyId::new(&db_id);
            let p = Policy::parse(Some(policy_id), &content)
                .map_err(|e| AuthServiceError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())))?;
            policies.push(p);
        }
        
        // Crear PolicySet desde todas las políticas de una vez
        let policy_set = PolicySet::from_policies(policies)
            .map_err(AuthServiceError::PolicyAdd)?;
        
        tracing::info!("✅ Loaded {} policies successfully", policy_count);
        Ok(policy_set)
    }

    async fn create_policy(&self, content: String) -> Result<String, AuthServiceError> {
        // Generar UUID único para la política (como AWS Verified Permissions)
        let policy_id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query(
            "INSERT INTO policies (id, content) VALUES ($1, $2)"
        )
        .bind(&policy_id)
        .bind(&content)
        .execute(&self.db)
        .await?;
        
        Ok(policy_id)
    }

    async fn update_policy(&self, id: String, content: String) -> Result<(), AuthServiceError> {
        let result = sqlx::query(
            "UPDATE policies SET content = $1 WHERE id = $2"
        )
        .bind(&content)
        .bind(&id)
        .execute(&self.db)
        .await?;
        
        if result.rows_affected() == 0 {
            return Err(AuthServiceError::PolicyIdNotFound);
        }
        
        Ok(())
    }

    async fn delete_policy(&self, id: String) -> Result<(), AuthServiceError> {
        let result = sqlx::query("DELETE FROM policies WHERE id = $1")
            .bind(&id)
            .execute(&self.db)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(AuthServiceError::PolicyIdNotFound);
        }
        
        Ok(())
    }

    async fn get_policy(&self, id: String) -> Result<Option<String>, AuthServiceError> {
        let record = sqlx::query("SELECT content FROM policies WHERE id = $1")
            .bind(&id)
            .fetch_optional(&self.db)
            .await?;
        
        Ok(record.map(|r| r.try_get("content").unwrap()))
    }

    async fn list_policies(&self) -> Result<Vec<(String, String)>, AuthServiceError> {
        let records = sqlx::query("SELECT id, content FROM policies")
            .fetch_all(&self.db)
            .await?;
        
        let mut policies = Vec::new();
        for record in records {
            let id: String = record.try_get("id")?;
            let content: String = record.try_get("content")?;
            policies.push((id, content));
        }
        
        Ok(policies)
    }
}

pub struct AuthorizationService {
    authorizer: Authorizer,
    schema: Arc<Schema>,
    policies: Arc<RwLock<PolicySet>>,
    adapter: Arc<dyn PolicyAdapter>,
}

impl AuthorizationService {
    pub async fn new(adapter: Arc<dyn PolicyAdapter>) -> Result<Self, AuthServiceError> {
        let schema_str = tokio::fs::read_to_string("cedar_schema.json").await?;
        let schema = Arc::new(Schema::from_json_str(&schema_str).map_err(|e| AuthServiceError::Schema(e))?);
        let policy_set = adapter.load_policies().await?;
        Ok(Self {
            authorizer: Authorizer::new(),
            schema,
            policies: Arc::new(RwLock::new(policy_set)),
            adapter,
        })
    }

    pub async fn is_authorized(&self, request: Request, entities: &Entities) -> Decision {
        let policies = self.policies.read().await;
        let response = self.authorizer.is_authorized(&request, &policies, entities);
        response.decision()
    }

    pub async fn create_policy(&self, content: String) -> Result<String, AuthServiceError> {
        let policy_id = self.adapter.create_policy(content).await?;
        self.reload_policies().await?;
        Ok(policy_id)
    }

    pub async fn update_policy(&self, id: String, content: String) -> Result<(), AuthServiceError> {
        self.adapter.update_policy(id, content).await?;
        self.reload_policies().await?;
        Ok(())
    }

    pub async fn delete_policy(&self, id: String) -> Result<(), AuthServiceError> {
        self.adapter.delete_policy(id).await?;
        self.reload_policies().await?;
        Ok(())
    }

    pub async fn get_policy(&self, id: String) -> Result<Option<String>, AuthServiceError> {
        self.adapter.get_policy(id).await
    }

    pub async fn list_policies(&self) -> Result<Vec<(String, String)>, AuthServiceError> {
        self.adapter.list_policies().await
    }

    async fn reload_policies(&self) -> Result<(), AuthServiceError> {
        let new_policies = self.adapter.load_policies().await?;
        let mut policies = self.policies.write().await;
        *policies = new_policies;
        Ok(())
    }
}
