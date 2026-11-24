//! Domain modeling for Families

use crate::uuid::{AuthorityId, FamilyId};

/// A CA Family tree.
pub struct Family {
    pub id: FamilyId,
    pub name: String,
    pub root_authority_id: AuthorityId,
    pub status: FamilyStatus,
}

/// The current status of a [Family].
pub enum FamilyStatus {
    /// The family is active and can issue certificates.
    Active,

    /// The family is suspended and cannot issue certificates.
    Suspended,

    /// The family is archived and cannot issue certificates.
    Archived,
}
