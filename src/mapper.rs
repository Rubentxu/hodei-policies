use cedar_policy::{Context, Request, Entities};
use serde_json::Value as JsonValue;
use thiserror::Error;
use crate::hodei_provider::{RuntimeHodeiActionMapper, RuntimeHodeiEntityMapper};

#[derive(Debug, Error)]
pub enum MapperError {
    #[error("Error de request Cedar: {0}")] Request(String),
    #[error("Recurso requerido pero no encontrado")] ResourceNotFound,
    #[error("La acción indica un payload de recurso, pero no se pudo generar")] VirtualResourceError,
    #[error("Error al construir contexto Cedar: {0}")] ContextError(String),
}

pub struct HodeiMapperService;

impl HodeiMapperService {
    /// Construye un paquete de autorización Cedar completamente genérico.
    /// 
    /// # Parámetros
    /// - `principal`: Cualquier entidad que implemente `RuntimeHodeiEntityMapper` (User, Service, etc.)
    /// - `action`: Cualquier acción que implemente `RuntimeHodeiActionMapper`
    /// - `resource_from_db`: Recurso opcional cargado de la base de datos (si no es una acción de creación)
    /// - `request_context`: Contexto de la solicitud (puede ser cualquier tipo que implemente Any)
    /// - `cedar_context_data`: Datos adicionales para el contexto Cedar (opcional)
    /// 
    /// # Retorna
    /// Un tuple con (Request, Entities) listo para ser evaluado por Cedar
    pub fn build_auth_package<P, A, R, C>(
        principal: &P,
        action: &A,
        resource_from_db: Option<&R>,
        request_context: &C,
        cedar_context_data: Option<JsonValue>,
    ) -> Result<(Request, Entities), MapperError>
    where
        P: RuntimeHodeiEntityMapper + Clone,
        A: RuntimeHodeiActionMapper,
        R: RuntimeHodeiEntityMapper + Clone,
        C: std::any::Any,
    {
        let principal_entity = principal.to_cedar_entity();
        
        // Determinar si la acción crea un recurso virtual o usa uno existente
        let (resource_euid, entities_vec) = 
            if let Some(virtual_resource) = action.get_payload_as_virtual_entity(request_context) {
                // Acción de creación: usar entidad virtual del payload
                let euid = virtual_resource.uid().clone();
                (euid, vec![principal_entity, virtual_resource])
            } else {
                // Acción sobre recurso existente: cargar de BD
                let resource = resource_from_db.ok_or(MapperError::ResourceNotFound)?;
                let resource_entity = resource.to_cedar_entity();
                (resource.to_cedar_euid(), vec![principal_entity, resource_entity])
            };

        // Construir entidades Cedar
        let entities = Entities::from_entities(entities_vec, None)
            .map_err(|e| MapperError::Request(format!("Error al crear entidades: {}", e)))?;
        
        // Construir contexto Cedar con datos opcionales
        let context_json = cedar_context_data.unwrap_or_else(|| serde_json::json!({}));
        let cedar_context = Context::from_json_value(context_json, None)
            .map_err(|e| MapperError::ContextError(e.to_string()))?;
        
        // Construir request Cedar
        let request = Request::new(
            principal.to_cedar_euid(),
            action.to_cedar_action_euid(),
            resource_euid,
            cedar_context,
            None,
        ).map_err(|e| MapperError::Request(e.to_string()))?;

        Ok((request, entities))
    }
    
    /// Versión simplificada para casos comunes donde solo se necesita el contexto básico
    pub fn build_auth_package_simple<P, A, R, C>(
        principal: &P,
        action: &A,
        resource_from_db: Option<&R>,
        request_context: &C,
    ) -> Result<(Request, Entities), MapperError>
    where
        P: RuntimeHodeiEntityMapper + Clone,
        A: RuntimeHodeiActionMapper,
        R: RuntimeHodeiEntityMapper + Clone,
        C: std::any::Any,
    {
        Self::build_auth_package(principal, action, resource_from_db, request_context, None)
    }
}
