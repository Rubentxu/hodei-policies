pub mod api;

pub use hodei_kernel;
pub use hodei_provider_derive::{HodeiAction, HodeiEntity};
pub use inventory;

pub use api::{EntitySchemaFragment, ActionSchemaFragment, RuntimeHodeiEntityMapper, RuntimeHodeiActionMapper};

inventory::collect!(api::EntitySchemaFragment);
inventory::collect!(api::ActionSchemaFragment);