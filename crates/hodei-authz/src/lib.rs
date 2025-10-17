pub mod api;
pub mod traits;

pub use hodei_hrn;
pub use hodei_derive::{HodeiAction, HodeiEntity};
pub use inventory;

pub use api::{EntitySchemaFragment, ActionSchemaFragment, RuntimeHodeiEntityMapper, RuntimeHodeiActionMapper};
pub use traits::{PolicyStore, CacheInvalidation, PolicyStoreError, CacheError};

inventory::collect!(api::EntitySchemaFragment);
inventory::collect!(api::ActionSchemaFragment);