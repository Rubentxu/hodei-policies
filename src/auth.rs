use cedar_policy::{Authorizer, Decision, Entities, Policy, PolicySet, Request, Schema};
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
    async fn add_policy(&self, id: String, content: String) -> Result<(), AuthServiceError>;
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
        let records = sqlx::query("SELECT content FROM policies")
            .fetch_all(&self.db)
            .await?;
        let mut policy_set = PolicySet::new();
        for record in records {
            let content: String = record.try_get("content")?;
            let p = content.parse().map_err(AuthServiceError::PolicyParse)?;
            policy_set.add(p)?;
        }
        Ok(policy_set)
    }

    async fn add_policy(&self, id: String, content: String) -> Result<(), AuthServiceError> {
        sqlx::query(
            "INSERT INTO policies (id, content) VALUES ($1, $2) ON CONFLICT (id) DO UPDATE SET content = $2"
        )
        .bind(&id)
        .bind(&content)
        .execute(&self.db)
        .await?;
        Ok(())
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
        self.authorizer.is_authorized(&request, &policies, entities).decision()
    }

    pub async fn add_policy(&self, id: String, content: String) -> Result<(), AuthServiceError> {
        let policy: Policy = content.parse().map_err(AuthServiceError::PolicyParse)?;
        self.adapter.add_policy(id, content.clone()).await?;
        let mut policies = self.policies.write().await;
        policies.add(policy)?;
        Ok(())
    }
}
