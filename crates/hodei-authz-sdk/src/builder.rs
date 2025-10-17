//! Builder pattern para configurar HodeiAuthService fácilmente

use crate::schema::{auto_discover_schema, SchemaError};
use cedar_policy::{Authorizer, PolicySet, Schema};
use hodei_authz::{CacheInvalidation, PolicyStore};
use hodei_authz_postgres::PostgresPolicyStore;
use hodei_authz_redis::RedisCacheInvalidation;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Error al construir el servicio
#[derive(Debug, thiserror::Error)]
pub enum BuildError {
    #[error("PostgreSQL pool is required")]
    MissingPostgres,
    
    #[error("Redis URL is required")]
    MissingRedis,
    
    #[error("Schema error: {0}")]
    Schema(#[from] SchemaError),
    
    #[error("Policy store error: {0}")]
    PolicyStore(String),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("Migration error: {0}")]
    Migration(String),
}

/// Servicio de autorización completo
pub struct HodeiAuthService {
    pub(crate) policy_store: Arc<PostgresPolicyStore>,
    pub(crate) cache_invalidation: Arc<RedisCacheInvalidation>,
    pub(crate) authorizer: Authorizer,
    pub(crate) schema: Arc<Schema>,
    pub(crate) policy_set: Arc<RwLock<PolicySet>>,
}

/// Builder para HodeiAuthService
pub struct HodeiAuthServiceBuilder {
    postgres_pool: Option<PgPool>,
    redis_url: Option<String>,
    schema: Option<Schema>,
    auto_migrate: bool,
}

impl Default for HodeiAuthServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HodeiAuthServiceBuilder {
    /// Crea un nuevo builder
    pub fn new() -> Self {
        Self {
            postgres_pool: None,
            redis_url: None,
            schema: None,
            auto_migrate: true,
        }
    }
    
    /// Configura el pool de PostgreSQL
    pub fn with_postgres(mut self, pool: PgPool) -> Self {
        self.postgres_pool = Some(pool);
        self
    }
    
    /// Configura la URL de Redis
    pub fn with_redis(mut self, url: impl Into<String>) -> Self {
        self.redis_url = Some(url.into());
        self
    }
    
    /// Auto-descubre el schema usando inventory
    ///
    /// Esto recolecta todos los EntitySchemaFragment y ActionSchemaFragment
    /// registrados por los derives HodeiEntity y HodeiAction.
    pub fn auto_discover_schema(mut self) -> Result<Self, SchemaError> {
        self.schema = Some(auto_discover_schema()?);
        Ok(self)
    }
    
    /// Usa un schema personalizado
    pub fn with_schema(mut self, schema: Schema) -> Self {
        self.schema = Some(schema);
        self
    }
    
    /// Deshabilita las migraciones automáticas
    pub fn without_auto_migrate(mut self) -> Self {
        self.auto_migrate = false;
        self
    }
    
    /// Construye el servicio
    pub async fn build(self) -> Result<HodeiAuthService, BuildError> {
        // Validar configuración
        let pool = self.postgres_pool.ok_or(BuildError::MissingPostgres)?;
        let redis_url = self.redis_url.ok_or(BuildError::MissingRedis)?;
        let schema = self.schema.ok_or_else(|| {
            SchemaError::InvalidStructure(
                "Schema is required. Call auto_discover_schema() or with_schema()".to_string()
            )
        })?;
        
        // Setup policy store
        let policy_store = PostgresPolicyStore::new(pool);
        
        if self.auto_migrate {
            policy_store
                .migrate()
                .await
                .map_err(|e| BuildError::Migration(e.to_string()))?;
            tracing::info!("✅ Database migrations completed");
        }
        
        // Setup cache invalidation
        let cache_invalidation = RedisCacheInvalidation::new(&redis_url)
            .await
            .map_err(|e| BuildError::Cache(e.to_string()))?;
        tracing::info!("✅ Redis cache connected");
        
        // Load policies
        let policy_set = Self::load_initial_policies(&policy_store).await?;
        tracing::info!("✅ Policies loaded");
        
        let authorizer = Authorizer::new();
        
        Ok(HodeiAuthService {
            policy_store: Arc::new(policy_store),
            cache_invalidation: Arc::new(cache_invalidation),
            authorizer,
            schema: Arc::new(schema),
            policy_set: Arc::new(RwLock::new(policy_set)),
        })
    }
    
    /// Carga las políticas iniciales
    async fn load_initial_policies(
        store: &PostgresPolicyStore,
    ) -> Result<PolicySet, BuildError> {
        store
            .load_all_policies()
            .await
            .map_err(|e| BuildError::PolicyStore(e.to_string()))
    }
}

impl HodeiAuthService {
    /// Crea un nuevo builder
    pub fn builder() -> HodeiAuthServiceBuilder {
        HodeiAuthServiceBuilder::new()
    }
    
    /// Obtiene el schema
    pub fn schema(&self) -> &Schema {
        &self.schema
    }
    
    /// Recarga las políticas
    pub async fn reload_policies(&self) -> Result<(), BuildError> {
        let new_policy_set = self
            .policy_store
            .load_all_policies()
            .await
            .map_err(|e| BuildError::PolicyStore(e.to_string()))?;
        
        let mut policy_set = self.policy_set.write().await;
        *policy_set = new_policy_set;
        
        Ok(())
    }
    
    /// Invalida el caché
    pub async fn invalidate_cache(&self) -> Result<(), BuildError> {
        self.cache_invalidation
            .invalidate_policies()
            .await
            .map_err(|e| BuildError::Cache(e.to_string()))
    }
}
