pub mod api;

mod internal;
mod features;

pub use internal::domain::{RequestContext, User, Document, Artifact};
pub use features::document_management::dto::{DocumentCommand, DocumentCreatePayload, DocumentUpdatePayload};
pub use features::artifact_management::dto::{ArtifactCommand, ArtifactCreatePayload, ArtifactUpdatePayload};

#[cfg(feature = "schema-discovery")]
#[used]
pub static HODEI_DOMAIN_INVENTORY_ANCHOR: fn() = || {};