use cedar_policy::{Entity, EntityUid, RestrictedExpression};
use hodei_kernel::Hrn;

pub(crate) use
pub use hodei_provider_derive::{HodeiAction, HodeiEntity};
pub use inventory;

pub(crate) struct EntitySchemaFragment {
    pub entity_type: &'static str,
    pub fragment_json: &'static str,
}

pub struct ActionSchemaFragment {
    pub name: &'static str,
    pub fragment_json: &'static str,
}

inventory::collect!(EntitySchemaFragment);
inventory::collect!(ActionSchemaFragment);

pub(crate) trait {
    fn hodei_type_name(&self) -> &'static str;
    fn hodei_id(&self) -> String;
    fn hodei_hrn(&self) -> &Hrn;

    fn to_cedar_euid(&self) -> EntityUid {
        EntityUid::from_type_name_and_id(
            self.hodei_type_name().parse().unwrap(),
            self.hodei_hrn().to_string().parse().unwrap(),
        )
    }
    fn to_cedar_entity(&self) -> Entity;
}

pub trait RuntimeHodeiActionMapper {
    fn to_cedar_action_euid(&self) -> EntityUid;
    fn creates_resource_from_payload(&self) -> bool;
    fn get_payload_as_virtual_entity(&self, context: &dyn std::any::Any) -> Option<Entity>;
}
